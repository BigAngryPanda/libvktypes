//! Provide API to GPU command buffers

use ash::vk;

use crate::{dev, memory, compute, graphics};

use crate::on_error_ret;

use std::{ptr, cmp};
use std::iter::Iterator;
use std::marker::PhantomData;
use std::sync::Arc;
use std::fmt;

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

pub struct PoolCfg {
    pub queue_index: u32,
}

#[derive(Debug)]
pub enum PoolError {
    /// Failed to
    /// [create](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateCommandPool.html)
    /// command pool
    Creating
}

/// All command buffers are allocated from `Pool`
pub struct Pool {
    i_core: Arc<dev::Core>,
    i_pool: vk::CommandPool
}

impl Pool {
    pub fn new(dev: &dev::Device, pool_type: &PoolCfg) -> Result<Pool, PoolError> {
        let pool_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags:  vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: pool_type.queue_index,
        };

        let cmd_pool = on_error_ret!(
            unsafe { dev.device().create_command_pool(&pool_info, None) },
            PoolError::Creating
        );

        Ok(
            Pool {
                i_core: dev.core().clone(),
                i_pool: cmd_pool
            }
        )
    }

    /// Allocate new command buffer
    pub fn allocate<'a>(&'a self) -> Result<Buffer<'a>, BufferError> {
        let cmd_buff_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_pool: self.i_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: 1,
        };

        let cmd_buffers = on_error_ret!(
            unsafe { self.i_core.device().allocate_command_buffers(&cmd_buff_info) },
            BufferError::Creating
        );

        let cmd_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            flags:  vk::CommandBufferUsageFlags::empty(),
            p_inheritance_info: ptr::null()
        };

        on_error_ret!(
            unsafe { self.i_core.device().begin_command_buffer(cmd_buffers[0], &cmd_begin_info) },
            BufferError::Begin
        );

        Ok(
            Buffer {
                i_buffer: cmd_buffers[0],
                i_pool: self,
            }
        )
    }

    #[doc(hidden)]
    fn device(&self) -> &ash::Device {
        self.i_core.device()
    }
}

impl fmt::Debug for Pool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pool")
        .field("i_core", &self.i_core)
        .field("i_pool", &(&self.i_pool as *const vk::CommandPool))
        .finish()
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        unsafe {
            self.i_core.device()
                .destroy_command_pool(
                    self.i_pool, self.i_core.allocator()
                );
        }
    }
}

#[derive(Debug)]
pub enum BufferError {
    /// Failed to
    /// [allocate](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkAllocateCommandBuffers.html)
    /// buffer
    Creating,
    /// Failed to
    /// [initialize](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkBeginCommandBuffer.html)
    /// buffer
    Begin,
    /// Failed to
    /// [complete](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkBeginCommandBuffer.html)
    /// buffer
    Commit
}

/// Buffer in which you can write commands
///
/// Note: this buffer is not ready for execution "as is"
///
/// For that you have to complete buffer via (`commit`)[crate::cmd::Buffer::commit]
pub struct Buffer<'a> {
    i_pool: &'a Pool,
    i_buffer: vk::CommandBuffer
}

impl<'a> Buffer<'a> {
    /// Modify buffer into executable
    ///
    /// Original buffer will not be available
    pub fn commit(self) -> Result<ExecutableBuffer<'a>, BufferError> {
        let dev = self.i_pool.device();

        on_error_ret!(
            unsafe { dev.end_command_buffer(self.i_buffer) },
            BufferError::Commit
        );

        Ok(
            ExecutableBuffer {
                i_buffer: self.i_buffer,
                _marker: PhantomData
            }
        )
    }

    /// Bind specifically *compute* pipeline
    ///
    /// For graphics see [`bind_graphics_pipeline`](Buffer::bind_graphics_pipeline)
    pub fn bind_compute_pipeline(&self, pipe: &compute::Pipeline) {
        let dev = self.i_pool.device();

        unsafe {
            dev.cmd_bind_pipeline(
                self.i_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipe.pipeline()
            );

            dev.cmd_bind_descriptor_sets(
                self.i_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipe.pipeline_layout(),
                0,
                &[pipe.descriptor_set()],
                &[]
            );
        }
    }

    // Copy `src` buffer into `dst`
    ///
    /// If `dst` has less capacity then copy only first (`dst.size()`)[crate::memory::View::size()] bytes
    ///
    /// If `src` has less capacity then rest of the `dst` memory will be left intact
    pub fn copy_memory(&self, src: &memory::View, dst: &memory::View) {
        let dev = self.i_pool.device();

        let copy_info = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: cmp::min(src.size(), dst.size()),
        };

        unsafe {
            dev.cmd_copy_buffer(self.i_buffer, src.buffer(), dst.buffer(), &[copy_info]);
        }
    }

    /// Dispatch work groups
    pub fn dispatch(&self, x: u32, y: u32, z: u32) {
        let dev = self.i_pool.device();

        unsafe {
            dev.cmd_dispatch(self.i_buffer, x, y, z)
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
    pub fn set_barrier(&mut self,
        mem: &memory::View,
        src_type: AccessType,
        dst_type: AccessType,
        src_stage: PipelineStage,
        dst_stage: PipelineStage)
    {
        let dev = self.i_pool.device();

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
                self.i_buffer,
                src_stage,
                dst_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[mem_barrier],
                &[]
            )
        }
    }

    /// Update push constatnts with raw data
    pub fn update_push_constants(&self, pipe: &compute::Pipeline, data: &[u8]) {
        let dev = self.i_pool.device();

        unsafe {
            dev.cmd_push_constants(
                self.i_buffer, pipe.pipeline_layout(), vk::ShaderStageFlags::COMPUTE, 0, data
            )
        }
    }

    /// Begin render pass with selected framebuffer
    ///
    /// Must be ended with [`end_render_pass`](crate::cmd::Buffer::end_render_pass)
    pub fn begin_render_pass(&self, rp: &graphics::RenderPass, fb: &memory::Framebuffer) {
        let dev = self.i_pool.device();

        let clear_value = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                }
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                }
            }
        ];

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
            clear_value_count: clear_value.len() as u32,
            p_clear_values: clear_value.as_ptr(),
        };

        unsafe {
            dev.cmd_begin_render_pass(self.i_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE)
        };
    }

    /// Update vertex bindings
    ///
    /// Updating starts from **first** binding
    pub fn bind_vertex_buffers(&self, buffers: &[memory::View]) {
        let dev = self.i_pool.device();

        let vertex_buffers: Vec<vk::Buffer> = buffers.iter().map(|x| x.buffer()).collect();
        let offsets: Vec<vk::DeviceSize> = vec![0; vertex_buffers.len()];

        unsafe {
            dev.cmd_bind_vertex_buffers(self.i_buffer, 0, vertex_buffers.as_slice(), offsets.as_slice())
        }
    }

    /// Bind specifically *graphics* pipeline
    ///
    /// For graphics see [`bind_compute_pipeline`](Buffer::bind_compute_pipeline)
    pub fn bind_graphics_pipeline(&self, pipe: &graphics::Pipeline) {
        let dev = self.i_pool.device();

        unsafe {
            dev.cmd_bind_pipeline(self.i_buffer, vk::PipelineBindPoint::GRAPHICS, pipe.pipeline())
        }
    }

    /// Enable resource usage for the `pipeline`
    ///
    /// `offsets` for now has no effect so leave it as `&[]`
    ///
    /// Note: do not confuse with [`update`](graphics::Pipeline::update) method
    pub fn bind_resources(&self, pipe: &graphics::Pipeline, offsets: &[u32]) {
        unsafe {
            self
            .i_pool
            .device()
            .cmd_bind_descriptor_sets(
                self.i_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipe.layout(),
                0,
                pipe.descriptor_set(),
                offsets
            );
        }
    }

    /// Add `vkCmdDraw` call to the buffer
    ///
    /// About args see [more](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCmdDraw.html)
    pub fn draw(&self, vc: u32, ic: u32, fv: u32, fi: u32) {
        let dev = self.i_pool.device();

        unsafe {
            dev.cmd_draw(self.i_buffer, vc, ic, fv, fi);
        }
    }

    /// End render pass
    ///
    /// Must be after [`begin_render_pass`](crate::cmd::Buffer::begin_render_pass)
    pub fn end_render_pass(&self) {
        let dev = self.i_pool.device();

        unsafe {
            dev.cmd_end_render_pass(self.i_buffer);
        }
    }
}

impl<'a> fmt::Debug for Buffer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Buffer")
        .field("i_pool", self.i_pool)
        .field("i_buffer", &self.i_buffer)
        .finish()
    }
}

/// Buffer which is ready for execution
pub struct ExecutableBuffer<'a> {
    i_buffer: vk::CommandBuffer,
    _marker: PhantomData<&'a Pool>
}

#[doc(hidden)]
impl<'a> ExecutableBuffer<'a> {
    pub fn buffer(&self) -> &vk::CommandBuffer {
        &self.i_buffer
    }
}

impl<'a> fmt::Debug for ExecutableBuffer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Buffer")
        .field("i_buffer", &self.i_buffer)
        .finish()
    }
}