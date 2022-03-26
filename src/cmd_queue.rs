//! Provide API to execute commands on GPU

use ash::vk;

use crate::logical_device::LogicalDevice;
use crate::memory::Memory;
use crate::pipeline::ComputePipeline;

use crate::on_error;

use std::ptr;

// TODO make this behaviour safe
// By returning CompleteComputeQueue after submit?

/// Represents command buffer that will be executed on GPU
///
/// To add command call `cmd_*` method
///
/// Call `submit` to **complete** queue
///
/// 'ComputeQueue is completed' means you **cannot** add more commands to queue, only execute
///
/// Before execution (by calling `exec` method) you **must** complete queue
/// (see [ComputeQueue::submit])
pub struct ComputeQueue<'a> {
	i_ldevice: &'a LogicalDevice<'a>,
	i_pool:    vk::CommandPool,
	i_buffer:  vk::CommandBuffer,
}

#[derive(Debug)]
pub enum ComputeQueueError {
	CommandPool,
	CommandBuffers,
	BufferInit,
	Submit,
	Fence,
	Execution,
	Timeout
}

/// AccessType specifies memory access
///
#[doc = "Ash documentation about possible values <https://docs.rs/ash/latest/ash/vk/struct.AccessFlags.html>"]
///
#[doc = "Vulkan documentation <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkAccessFlagBits.html>"]
pub type AccessType = vk::AccessFlags;

/// PipelineStage specifies single pipeline stage
///
#[doc = "Ash documentation about possible values <https://docs.rs/ash/latest/ash/vk/struct.PipelineStageFlags.html>"]
///
#[doc = "Vulkan documentation <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineStageFlagBits.html>"]
pub type PipelineStage = vk::PipelineStageFlags;

// TODO more buffers?
// TODO check if we submit buffer before execution
impl<'a> ComputeQueue<'a> {
	pub fn new(dev: &'a LogicalDevice) -> Result<ComputeQueue<'a>, ComputeQueueError> {
		let device = dev.device();

		let pool_info = vk::CommandPoolCreateInfo {
			s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
			p_next: ptr::null(),
			flags:  vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
			queue_family_index: dev.queue_index()
		};

		let cmd_pool = on_error!(
			unsafe { device.create_command_pool(&pool_info, None) },
			return Err(ComputeQueueError::CommandPool)
		);

		let cmd_buff_info = vk::CommandBufferAllocateInfo {
			s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
			p_next: ptr::null(),
			command_pool: cmd_pool,
			level: vk::CommandBufferLevel::PRIMARY,
			command_buffer_count: 1,
		};

		let cmd_buffers = on_error!(
			unsafe { device.allocate_command_buffers(&cmd_buff_info) },
			return Err(ComputeQueueError::CommandBuffers)
		);

		let cmd_begin_info = vk::CommandBufferBeginInfo {
			s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
			p_next: ptr::null(),
			flags:  vk::CommandBufferUsageFlags::empty(),
			p_inheritance_info: ptr::null()
		};

		on_error!(
			unsafe { device.begin_command_buffer(cmd_buffers[0], &cmd_begin_info) },
			return Err(ComputeQueueError::BufferInit)
		);

		Ok(
			ComputeQueue {
				i_ldevice: dev,
				i_pool:    cmd_pool,
				i_buffer:  cmd_buffers[0] // for now only one buffer is supported
			}
		)
	}

	/// Copy `src` buffer into `dst`
	///
	/// If `dst` has less capacity then copy only first [Memory::size()] bytes
	///
	/// If `src` has less capacity then rest of the `dst` memory will be left intact
	pub fn cmd_copy(&self, src: &Memory, dst: &Memory)  {
		use std::cmp;

		let copy_info = vk::BufferCopy {
			src_offset: 0,
			dst_offset: 0,
			size: cmp::min(src.size(), dst.size()),
		};

		unsafe {
			self.i_ldevice.device().cmd_copy_buffer(self.i_buffer, src.buffer(), dst.buffer(), &[copy_info]);
		}
	}

	// TODO can we infer AccessType and PipelineStage from buffer type?
	// I think not
	// Add usage type to Memory?

	/// Set *buffer* memory barrier
	/// ([see more](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkBufferMemoryBarrier.html))
	///
	/// `src` is what should be before barrier (e.g. write to memory)
	///
	/// `dst` is what should be after barrier (e.g. read)
	///
	/// For more types see [AccessType]
	pub fn cmd_set_barrier(&self,
		mem: &Memory,
		src_type: AccessType,
		dst_type: AccessType,
		src_stage: PipelineStage,
		dst_stage: PipelineStage)
	{
		let mem_barrier = vk::BufferMemoryBarrier {
			s_type: vk::StructureType::BUFFER_MEMORY_BARRIER,
			p_next: ptr::null(),
			src_access_mask: src_type,
			dst_access_mask: dst_type,
			src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
			dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
			buffer: mem.buffer(),
			offset: 0,
			size: vk::WHOLE_SIZE,
		};

		unsafe {
			self.i_ldevice.device().cmd_pipeline_barrier(
				self.i_buffer, src_stage, dst_stage, vk::DependencyFlags::empty(), &[], &[mem_barrier], &[]
			)
		}
	}

	pub fn cmd_bind_pipeline(&self, pipe: &ComputePipeline) {
		let device = self.i_ldevice.device();

		unsafe {
			device.cmd_bind_pipeline(self.i_buffer, vk::PipelineBindPoint::COMPUTE, pipe.pipeline());

			device.cmd_bind_descriptor_sets(
				self.i_buffer, vk::PipelineBindPoint::COMPUTE, pipe.pipeline_layout(), 0, &[pipe.descriptor_set()], &[]
			)
		}
	}

	/// Dispatch work groups
	pub fn dispatch(&self, x: u32, y: u32, z: u32) {
		unsafe {
			self.i_ldevice.device().cmd_dispatch(self.i_buffer, x, y, z)
		}
	}

	pub fn submit(&self) -> Result<(), ComputeQueueError> {
		on_error!(
			unsafe { self.i_ldevice.device().end_command_buffer(self.i_buffer) },
			return Err(ComputeQueueError::Submit)
		);

		Ok(())
	}

	// TODO bug! we have to destroy fence if any error occured after fence creating
	pub fn exec(&self, wait_stage: PipelineStage, timeout: u64) -> Result<(), ComputeQueueError> {
		let device = self.i_ldevice.device();

		let fence_info = vk::FenceCreateInfo {
			s_type: vk::StructureType::FENCE_CREATE_INFO,
			p_next: ptr::null(),
			flags:  vk::FenceCreateFlags::empty()
		};

		let fence = on_error!(
			unsafe { device.create_fence(&fence_info, None) },
			return Err(ComputeQueueError::Fence)
		);

		let submit_info = vk::SubmitInfo {
			s_type: vk::StructureType::SUBMIT_INFO,
			p_next: ptr::null(),
			wait_semaphore_count: 0,
			p_wait_semaphores: ptr::null(),
			p_wait_dst_stage_mask: &wait_stage,
			command_buffer_count: 1,
			p_command_buffers: &self.i_buffer,
			signal_semaphore_count: 0,
			p_signal_semaphores: ptr::null(),
		};

		on_error!(
			unsafe { device.queue_submit(self.i_ldevice.queue(), &[submit_info], fence) },
			return Err(ComputeQueueError::Fence)
		);

		on_error!(
			unsafe { device.wait_for_fences(&[fence], true, timeout) },
			return Err(ComputeQueueError::Timeout)
		);

		unsafe { device.destroy_fence(fence, None) };

		Ok(())
	}
}

impl<'a> Drop for ComputeQueue<'a> {
	fn drop(&mut self) {
		let device = self.i_ldevice.device();

		unsafe {
			device.destroy_command_pool(self.i_pool, None)
		}
	}
}