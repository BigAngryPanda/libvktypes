//! Provide API to execute commands on GPU

use ash::vk;

use crate::{dev, memory, compute};

use crate::on_error_ret;

use std::{ptr, cmp};
use std::iter::Iterator;

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

pub enum Cmd<'a> {
    BindPipeline(&'a compute::Pipeline<'a>),
    UpdatePushConstants(&'a compute::Pipeline<'a>, &'a [u8]),
    CopyMemory(&'a memory::Memory<'a>, &'a memory::Memory<'a>),
    // memory, src_type, dst_type, src_stage, src_stage
    SetBarrier(&'a memory::Memory<'a>, AccessType, AccessType, PipelineStage, PipelineStage),
    // x, y, z
    Dispatch(u32, u32, u32),
}

pub struct CmdPoolType<'a> {
    pub device: &'a dev::Device<'a>
}

#[derive(Debug)]
pub enum CmdPoolError {
    CommandPool,
}

pub struct CmdPool<'a> {
    i_device: &'a dev::Device<'a>,
    i_pool: vk::CommandPool,
}

impl<'a> CmdPool<'a> {
    pub fn new(pool_type: &'a CmdPoolType) -> Result<CmdPool<'a>, CmdPoolError> {
        let device = pool_type.device.device();

        let pool_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags:  vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: pool_type.device.queue_index()
        };

        let cmd_pool = on_error_ret!(
            unsafe { device.create_command_pool(&pool_info, None) },
            CmdPoolError::CommandPool
        );

        Ok(
            CmdPool {
                i_device: pool_type.device,
                i_pool: cmd_pool
            }
        )
    }

    fn cmd_pool(&self) -> vk::CommandPool {
        self.i_pool
    }

    fn device(&self) -> &'a dev::Device {
        self.i_device
    }
}

impl<'a> Drop for CmdPool<'a> {
    fn drop(&mut self) {
        unsafe {
            self.i_device.device().destroy_command_pool(self.i_pool, None)
        }
    }
}

#[derive(Default)]
pub struct CmdBufferType<'a> {
    i_cmds: Vec<Cmd<'a>>
}

impl<'a> CmdBufferType<'a> {
    pub fn new() -> CmdBufferType<'a> {
        CmdBufferType::default()
    }

    /// Dispatch work groups
    pub fn dispatch(&mut self, x: u32, y: u32, z: u32) {
        self.i_cmds.push(Cmd::Dispatch(x, y, z));
    }

    pub fn bind_pipeline(&mut self, pipe: &'a compute::Pipeline) {
        self.i_cmds.push(Cmd::BindPipeline(pipe));
    }

    pub fn update_push_constants(&mut self, pipe: &'a compute::Pipeline, data: &'a [u8]) {
        self.i_cmds.push(Cmd::UpdatePushConstants(pipe, data));
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
    pub fn set_barrier(&mut self,
        mem: &'a memory::Memory,
        src_type: AccessType,
        dst_type: AccessType,
        src_stage: PipelineStage,
        dst_stage: PipelineStage)
    {
        self.i_cmds.push(Cmd::SetBarrier(mem, src_type, dst_type, src_stage, dst_stage));
    }

    /// Copy `src` buffer into `dst`
    ///
    /// If `dst` has less capacity then copy only first [crate::memory::Memory::size()] bytes
    ///
    /// If `src` has less capacity then rest of the `dst` memory will be left intact
    pub fn copy(&mut self, src: &'a memory::Memory, dst: &'a memory::Memory)  {
        self.i_cmds.push(Cmd::CopyMemory(src, dst));
    }

    /// Return iterator over internal buffer
    pub fn iter(&self) -> impl Iterator<Item = &Cmd<'a>> {
        self.i_cmds.iter()
    }
}

pub struct ComputeQueueType<'a> {
    pub cmd_pool: &'a CmdPool<'a>,
    pub cmd_buffer: &'a CmdBufferType<'a>,
    pub queue_index: u32,
}

#[derive(Debug)]
pub enum ComputeQueueError {
    CommandBuffers,
    BufferInit,
    Commit,
    Fence,
    Queue,
    Execution,
    Timeout
}

pub struct ComputeQueue<'a> {
    i_cmd_pool: &'a CmdPool<'a>,
    i_queue: vk::Queue,
    i_cmd_buffer: vk::CommandBuffer,
}

impl<'a> ComputeQueue<'a> {
    pub fn commit(queue_type: &'a ComputeQueueType) -> Result<ComputeQueue<'a>, ComputeQueueError> {
        let dev = queue_type.cmd_pool.device();

        let cmd_buff_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_pool: queue_type.cmd_pool.cmd_pool(),
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: 1,
        };

        let cmd_buffers = on_error_ret!(
            unsafe { dev.device().allocate_command_buffers(&cmd_buff_info) },
            ComputeQueueError::CommandBuffers
        );

        let dev_queue: vk::Queue = unsafe {
            dev.device().get_device_queue(dev.queue_index(), queue_type.queue_index)
        };

        let result = ComputeQueue {
            i_cmd_pool: queue_type.cmd_pool,
            i_queue: dev_queue,
            i_cmd_buffer: cmd_buffers[0],
        };

        if result.fill_buffer(queue_type.cmd_buffer).is_ok() {
            Ok(result)
        }
        else {
            Err(ComputeQueueError::Commit)
        }
    }

    pub fn exec(&self, wait_stage: PipelineStage, timeout: u64) -> Result<(), ComputeQueueError> {
        let dev = self.i_cmd_pool.device().device();

        let fence_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags:  vk::FenceCreateFlags::empty()
        };

        let fence = on_error_ret!(
            unsafe { dev.create_fence(&fence_info, None) },
            ComputeQueueError::Fence
        );

        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: 0,
            p_wait_semaphores: ptr::null(),
            p_wait_dst_stage_mask: &wait_stage,
            command_buffer_count: 1,
            p_command_buffers: &self.i_cmd_buffer,
            signal_semaphore_count: 0,
            p_signal_semaphores: ptr::null(),
        };

        unsafe {
            if dev.queue_submit(self.i_queue, &[submit_info], fence).is_err() {
               dev.destroy_fence(fence, None);
               return Err(ComputeQueueError::Queue);
            }
        }

        unsafe {
            if dev.wait_for_fences(&[fence], true, timeout).is_err() {
               dev.destroy_fence(fence, None);
               return Err(ComputeQueueError::Timeout);
            }
        }

        unsafe { dev.destroy_fence(fence, None) };

        Ok(())
    }

    fn fill_buffer(&self, cmd_buffer: &CmdBufferType) -> Result<(), ComputeQueueError> {
        on_error_ret!(self.begin_buffer(), ComputeQueueError::BufferInit);

        for cmd in cmd_buffer.iter() {
            match cmd {
                Cmd::BindPipeline(pipe) => {
                    self.bind_pipeline(pipe);
                },
                Cmd::CopyMemory(src, dst) => {
                    self.copy_memory(src, dst);
                },
                Cmd::Dispatch(x, y, z) => {
                    self.dispatch(*x, *y, *z);
                },
                // memory, access before, access after, stage before, stage after
                Cmd::SetBarrier(m, ab, aa, sb, sa) => {
                    self.set_barrier(m, *ab, *aa, *sb, *sa);
                },
                Cmd::UpdatePushConstants(pipe, data) => {
                    self.update_push_constants(pipe, data);
                },
            }
        }

        self.end_buffer()
    }

    fn begin_buffer(&self) -> Result<(), ComputeQueueError> {
        let dev = self.i_cmd_pool.device().device();

        let cmd_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            flags:  vk::CommandBufferUsageFlags::empty(),
            p_inheritance_info: ptr::null()
        };

        on_error_ret!(
            unsafe { dev.begin_command_buffer(self.i_cmd_buffer, &cmd_begin_info) },
            ComputeQueueError::BufferInit
        );

        Ok(())
    }

    fn bind_pipeline(&self, pipe: &'a compute::Pipeline<'a>) {
        let dev = self.i_cmd_pool.device().device();

        unsafe {
            dev.cmd_bind_pipeline(
                self.i_cmd_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipe.pipeline()
            );

            dev.cmd_bind_descriptor_sets(
                self.i_cmd_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipe.pipeline_layout(),
                0,
                &[pipe.descriptor_set()],
                &[]
            );
        }
    }

    fn copy_memory(&self, src: &memory::Memory, dst: &memory::Memory) {
        let dev = self.i_cmd_pool.device().device();

        let copy_info = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: cmp::min(src.size(), dst.size()),
        };

        unsafe {
            dev.cmd_copy_buffer(self.i_cmd_buffer, src.buffer(), dst.buffer(), &[copy_info]);
        }
    }

    fn dispatch(&self, x: u32, y: u32, z: u32) {
        let dev = self.i_cmd_pool.device().device();

        unsafe {
            dev.cmd_dispatch(self.i_cmd_buffer, x, y, z)
        }
    }

    fn set_barrier(&self,
        mem: &memory::Memory,
        src_type: AccessType,
        dst_type: AccessType,
        src_stage: PipelineStage,
        dst_stage: PipelineStage)
    {
        let dev = self.i_cmd_pool.device().device();

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
            dev.cmd_pipeline_barrier(
                self.i_cmd_buffer,
                src_stage,
                dst_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[mem_barrier],
                &[]
            )
        }
    }

    fn update_push_constants(&self, pipe: &'a compute::Pipeline<'a>, data: &'a [u8]) {
        let dev = self.i_cmd_pool.device().device();

        unsafe {
            dev.cmd_push_constants(
                self.i_cmd_buffer, pipe.pipeline_layout(), vk::ShaderStageFlags::COMPUTE, 0, data
            )
        }
    }

    fn end_buffer(&self) -> Result<(), ComputeQueueError> {
        let dev = self.i_cmd_pool.device().device();

		on_error_ret!(
			unsafe { dev.end_command_buffer(self.i_cmd_buffer) },
			ComputeQueueError::Commit
		);

		Ok(())
    }
}