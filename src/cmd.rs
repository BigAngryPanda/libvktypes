//! Provide API to execute commands on GPU

use ash::vk;

use crate::{dev, memory, compute, graphics, sync};

use crate::{on_error_ret, data_ptr};

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
    BeginRenderPass(&'a graphics::RenderPass<'a>, &'a memory::Framebuffer<'a>),
    BindGraphicsPipeline(&'a graphics::Pipeline<'a>),
    Draw(u32, u32, u32, u32),
    EndRenderPass,
}

pub struct CmdPoolType<'a> {
    pub device: &'a dev::Device
}

#[derive(Debug)]
pub enum CmdPoolError {
    CommandPool,
}

pub struct CmdPool<'a> {
    i_device: &'a dev::Device,
    i_pool: vk::CommandPool,
}

impl<'a> CmdPool<'a> {
    pub fn new<'b>(pool_type: &'b CmdPoolType<'a>) -> Result<CmdPool<'a>, CmdPoolError> {
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

    pub fn cmd_pool(&self) -> vk::CommandPool {
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
pub struct CmdBuffer<'a> {
    i_cmds: Vec<Cmd<'a>>
}

impl<'a> CmdBuffer<'a> {
    pub fn new() -> CmdBuffer<'a> {
        CmdBuffer::default()
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

    /// Begin render pass with selected framebuffer
    ///
    /// Must be ended with [`end_render_pass`]
    pub fn begin_render_pass(&mut self, rp: &'a graphics::RenderPass<'a>, fb: &'a memory::Framebuffer<'a>) {
        self.i_cmds.push(Cmd::BeginRenderPass(rp, fb));
    }

    /// Bind graphics pipeline
    pub fn bind_graphics_pipeline(&mut self, pipe: &'a graphics::Pipeline) {
        self.i_cmds.push(Cmd::BindGraphicsPipeline(pipe));
    }

    /// Add `vkCmdDraw` call to the buffer
    ///
    /// About args see [more](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCmdDraw.html)
    pub fn draw(&mut self, vertex_count: u32, instance_count: u32, first_vert: u32, first_inst: u32) {
        self.i_cmds.push(Cmd::Draw(vertex_count, instance_count, first_vert, first_inst));
    }

    pub fn end_render_pass(&mut self) {
        self.i_cmds.push(Cmd::EndRenderPass);
    }

    /// Return iterator over internal buffer
    pub fn iter(&self) -> impl Iterator<Item = &Cmd<'a>> {
        self.i_cmds.iter()
    }
}

pub struct ExecInfo<'a, 'b> {
    pub wait_stage: PipelineStage,
    pub timeout: u64,
    pub wait: &'a [&'a sync::Semaphore<'b>],
    pub signal: &'a [&'a sync::Semaphore<'b>],
}

pub struct ComputeQueueType<'a> {
    pub cmd_pool: &'a CmdPool<'a>,
    pub cmd_buffer: &'a CmdBuffer<'a>,
    pub queue_index: u32,
}

#[derive(Debug)]
pub enum CompletedQueueError {
    CommandBuffers,
    BufferInit,
    Commit,
    Fence,
    Queue,
    Execution,
    Timeout
}

pub struct CompletedQueue<'a> {
    i_cmd_pool: &'a CmdPool<'a>,
    i_queue: vk::Queue,
    i_cmd_buffer: vk::CommandBuffer,
}

impl<'a> CompletedQueue<'a> {
    pub fn commit(queue_type: &'a ComputeQueueType) -> Result<CompletedQueue<'a>, CompletedQueueError> {
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
            CompletedQueueError::CommandBuffers
        );

        let dev_queue: vk::Queue = unsafe {
            dev.device().get_device_queue(dev.queue_index(), queue_type.queue_index)
        };

        let result = CompletedQueue {
            i_cmd_pool: queue_type.cmd_pool,
            i_queue: dev_queue,
            i_cmd_buffer: cmd_buffers[0],
        };

        if result.fill_buffer(queue_type.cmd_buffer).is_ok() {
            Ok(result)
        }
        else {
            Err(CompletedQueueError::Commit)
        }
    }

    pub fn exec(&self, info: &ExecInfo) -> Result<(), CompletedQueueError> {
        let dev = self.device();

        let fence_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags:  vk::FenceCreateFlags::empty()
        };

        let fence = on_error_ret!(
            unsafe { dev.create_fence(&fence_info, None) },
            CompletedQueueError::Fence
        );

        let wait_sems: Vec<vk::Semaphore> = info.wait.iter().map(|s| s.semaphore()).collect();
        let sign_sems: Vec<vk::Semaphore> = info.signal.iter().map(|s| s.semaphore()).collect();

        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: wait_sems.len() as u32,
            p_wait_semaphores: data_ptr!(wait_sems),
            p_wait_dst_stage_mask: &info.wait_stage,
            command_buffer_count: 1,
            p_command_buffers: &self.i_cmd_buffer,
            signal_semaphore_count: sign_sems.len() as u32,
            p_signal_semaphores: data_ptr!(sign_sems),
        };

        unsafe {
            if dev.queue_submit(self.i_queue, &[submit_info], fence).is_err() {
               dev.destroy_fence(fence, None);
               return Err(CompletedQueueError::Queue);
            }
        }

        unsafe {
            if dev.wait_for_fences(&[fence], true, info.timeout).is_err() {
               dev.destroy_fence(fence, None);
               return Err(CompletedQueueError::Timeout);
            }
        }

        unsafe { dev.destroy_fence(fence, None) };

        Ok(())
    }

    #[doc(hidden)]
    pub fn queue(&self) -> vk::Queue {
        self.i_queue
    }

    #[doc(hidden)]
    pub fn buffer(&self) -> vk::CommandBuffer {
        self.i_cmd_buffer
    }

    fn fill_buffer(&self, cmd_buffer: &CmdBuffer) -> Result<(), CompletedQueueError> {
        on_error_ret!(self.begin_buffer(), CompletedQueueError::BufferInit);

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
                Cmd::BeginRenderPass(rp, fb) => {
                    self.begin_render_pass(rp, fb);
                },
                Cmd::BindGraphicsPipeline(pipe) => {
                    self.bind_graphics_pipeline(pipe);
                },
                Cmd::Draw(vc, ic, fv, fi) => {
                    self.draw(*vc, *ic, *fv, *fi);
                },
                Cmd::EndRenderPass => {
                    self.end_render_pass();
                }
            }
        }

        self.end_buffer()
    }

    fn begin_buffer(&self) -> Result<(), CompletedQueueError> {
        let cmd_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            flags:  vk::CommandBufferUsageFlags::empty(),
            p_inheritance_info: ptr::null()
        };

        on_error_ret!(
            unsafe { self.device().begin_command_buffer(self.i_cmd_buffer, &cmd_begin_info) },
            CompletedQueueError::BufferInit
        );

        Ok(())
    }

    fn bind_pipeline(&self, pipe: &'a compute::Pipeline<'a>) {
        unsafe {
            self.device().cmd_bind_pipeline(
                self.i_cmd_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipe.pipeline()
            );

            self.device().cmd_bind_descriptor_sets(
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
        let copy_info = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: cmp::min(src.size(), dst.size()),
        };

        unsafe {
            self.device().cmd_copy_buffer(self.i_cmd_buffer, src.buffer(), dst.buffer(), &[copy_info]);
        }
    }

    fn dispatch(&self, x: u32, y: u32, z: u32) {
        unsafe {
            self.device().cmd_dispatch(self.i_cmd_buffer, x, y, z)
        }
    }

    fn set_barrier(&self,
        mem: &memory::Memory,
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
            self.device().cmd_pipeline_barrier(
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
        unsafe {
            self.device().cmd_push_constants(
                self.i_cmd_buffer, pipe.pipeline_layout(), vk::ShaderStageFlags::COMPUTE, 0, data
            )
        }
    }

    fn begin_render_pass<'b>(&self, rp: &'b graphics::RenderPass<'a>, fb: &'b memory::Framebuffer<'a>) {
        let clear_value:vk::ClearValue = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 0.0],
            }
        };

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_pass: rp.render_pass(),
            framebuffer: fb.framebuffer(),
            render_area: vk::Rect2D {
                offset: vk::Offset2D {
                    x: 0,
                    y: 0,
                },
                extent: fb.extent(),
            },
            clear_value_count: 1,
            p_clear_values: &clear_value,
        };

        unsafe {
            self.device().cmd_begin_render_pass(self.i_cmd_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE)
        };
    }

    fn bind_graphics_pipeline<'b>(&self, pipe: &'b graphics::Pipeline<'a>) {
        unsafe {
            self.device().cmd_bind_pipeline(self.i_cmd_buffer, vk::PipelineBindPoint::GRAPHICS, pipe.pipeline())
        }
    }

    fn draw(&self, vc: u32, ic: u32, fv: u32, fi: u32) {
        unsafe {
            self.device().cmd_draw(self.i_cmd_buffer, vc, ic, fv, fi);
        }
    }

    fn end_render_pass(&self) {
        unsafe {
            self.device().cmd_end_render_pass(self.i_cmd_buffer);
        }
    }

    fn end_buffer(&self) -> Result<(), CompletedQueueError> {
		on_error_ret!(
			unsafe { self.device().end_command_buffer(self.i_cmd_buffer) },
			CompletedQueueError::Commit
		);

		Ok(())
    }

    fn device(&self) -> &ash::Device {
        self.i_cmd_pool.device().device()
    }
}