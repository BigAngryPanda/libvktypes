//! Provide API to GPU command buffers

use ash::vk;

use crate::{dev, memory, compute, graphics};

use crate::on_error_ret;

use std::{ptr, cmp};
use std::iter::Iterator;
use std::sync::Arc;
use std::fmt;
use std::marker::PhantomData;

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

/// Special value for barriers to ignore specific queue family
pub const QUEUE_FAMILY_IGNORED: u32 = vk::QUEUE_FAMILY_IGNORED;

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

struct CorePool {
    i_core: Arc<dev::Core>,
    i_pool: vk::CommandPool
}

impl fmt::Debug for CorePool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pool")
        .field("i_core", &self.i_core)
        .field("i_pool", &(&self.i_pool as *const vk::CommandPool))
        .finish()
    }
}

impl Drop for CorePool {
    fn drop(&mut self) {
        unsafe {
            self.i_core.device()
                .destroy_command_pool(
                    self.i_pool, self.i_core.allocator()
                );
        }
    }
}

/// All command buffers are allocated from `Pool`
#[derive(Debug, Clone)]
pub struct Pool(Arc<CorePool>);

impl Pool {
    pub fn new(dev: &dev::Device, pool_type: &PoolCfg) -> Result<Pool, PoolError> {
        let pool_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags:  vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: pool_type.queue_index,
            _marker: PhantomData,
        };

        let cmd_pool = on_error_ret!(
            unsafe { dev.device().create_command_pool(&pool_info, None) },
            PoolError::Creating
        );

        Ok(Pool(
            Arc::new(CorePool {
            i_core: dev.core().clone(),
            i_pool: cmd_pool
            }
        )))
    }

    /// Allocate new command buffer
    pub fn allocate(&self) -> Result<Buffer, BufferError> {
        let cmd_buff_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_pool: self.0.i_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: 1,
            _marker: PhantomData,
        };

        let cmd_buffers = on_error_ret!(
            unsafe { self.0.i_core.device().allocate_command_buffers(&cmd_buff_info) },
            BufferError::Creating
        );

        let cmd_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            flags:  vk::CommandBufferUsageFlags::empty(),
            p_inheritance_info: ptr::null(),
            _marker: PhantomData,
        };

        on_error_ret!(
            unsafe { self.0.i_core.device().begin_command_buffer(cmd_buffers[0], &cmd_begin_info) },
            BufferError::Begin
        );

        Ok(
            Buffer {
                i_buffer: cmd_buffers[0],
                i_pool: self.clone(),
            }
        )
    }

    #[doc(hidden)]
    fn device(&self) -> &ash::Device {
        self.0.i_core.device()
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
pub struct Buffer {
    i_pool: Pool,
    i_buffer: vk::CommandBuffer
}

impl Buffer {
    /// Modify buffer into executable
    ///
    /// Original buffer will not be available
    pub fn commit(self) -> Result<ExecutableBuffer, BufferError> {
        let dev = self.i_pool.device();

        on_error_ret!(
            unsafe { dev.end_command_buffer(self.i_buffer) },
            BufferError::Commit
        );

        Ok(
            ExecutableBuffer {
                i_buffer: self.i_buffer,
                i_pool: self.i_pool,
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

    /// Copy `src` buffer into `dst`
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

    /// Copy `src` buffer into `dst`
    ///
    /// Function does not check size of the buffers
    ///
    /// `dst` image must has layout [`TRANSFER_DST_OPTIMAL`](memory::ImageLayout::TRANSFER_DST_OPTIMAL)
    /// or [`GENERAL`](memory::ImageLayout::GENERAL) on creation or via [barrier](Buffer::set_image_barrier)
    pub fn copy_buffer_to_image(&self, src: memory::View, dst: memory::ImageView) {
        let dev = self.i_pool.device();

        let copy_info = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: dst.subresource_layer(),
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
            image_extent: dst.extent(),
        };

        let transfer_layout = memory::ImageLayout::from_raw(
            (memory::ImageLayout::TRANSFER_DST_OPTIMAL).as_raw() | (memory::ImageLayout::GENERAL).as_raw()
        );

        unsafe {
            dev.cmd_copy_buffer_to_image(
                self.i_buffer,
                src.buffer(),
                dst.image(),
                transfer_layout,
                &[copy_info]);
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
        dst_stage: PipelineStage,
        src_queue_family: u32,
        dst_queue_family: u32)
    {
        let dev = self.i_pool.device();

        let mem_barrier = vk::BufferMemoryBarrier {
            s_type: vk::StructureType::BUFFER_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask: src_type,
            dst_access_mask: dst_type,
            src_queue_family_index: src_queue_family,
            dst_queue_family_index: dst_queue_family,
            buffer: mem.buffer(),
            offset: mem.offset(),
            size: mem.size(),
            _marker: PhantomData,
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

    /// Set image memory barrier
    /// ([see more](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkBufferMemoryBarrier.html))
    ///
    /// `src` is what should be before barrier (e.g. write to memory)
    ///
    /// `dst` is what should be after barrier (e.g. read)
    ///
    /// For more types see [AccessType]
    ///
    /// If you don't care for specific queue family use [`cmd::QUEUE_FAMILY_IGNORED`](QUEUE_FAMILY_IGNORED)
    pub fn set_image_barrier(&self,
        view: memory::ImageView,
        src_type: AccessType,
        dst_type: AccessType,
        src_layout: memory::ImageLayout,
        dst_layout: memory::ImageLayout,
        src_stage: PipelineStage,
        dst_stage: PipelineStage,
        src_queue_family: u32,
        dst_queue_family: u32)
    {
        let img_barrier = vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask: src_type,
            dst_access_mask: dst_type,
            old_layout: src_layout,
            new_layout: dst_layout,
            src_queue_family_index: src_queue_family,
            dst_queue_family_index: dst_queue_family,
            image: view.image(),
            subresource_range: view.subresource_range(),
            _marker: PhantomData,
        };

        unsafe {
            self.i_pool.device()
            .cmd_pipeline_barrier(
                self.i_buffer,
                src_stage,
                dst_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[img_barrier]
            )
        };

        view.set_layout(dst_layout);
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
            _marker: PhantomData,
        };

        unsafe {
            dev.cmd_begin_render_pass(self.i_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE)
        };
    }

    /// Update vertex bindings
    ///
    /// Updating starts from **first** binding
    pub fn bind_vertex_buffers(&self, buffers: &[graphics::VertexView]) {
        let dev = self.i_pool.device();

        let vertex_buffers: Vec<vk::Buffer> = buffers.iter().map(|x| x.buffer()).collect();
        let offsets: Vec<vk::DeviceSize> = buffers.iter().map(|x| x.offset() as u64).collect();

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
    /// Each element of `offsets` must be multiple of [`hw::ubo_offset`](crate::hw::HWDevice::ubo_offset)
    ///
    /// See [more](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCmdBindDescriptorSets.html)
    ///
    /// If you do not care about `offsets` leave it as `&[]`
    pub fn bind_resources(&self, pipe: &graphics::Pipeline, res: &graphics::PipelineDescriptor, offsets: &[u32]) {
        unsafe {
            self
            .i_pool
            .device()
            .cmd_bind_descriptor_sets(
                self.i_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipe.layout(),
                0,
                res.descriptor_sets(),
                offsets
            );
        }
    }

    /// Bind index buffer
    pub fn bind_index_buffer(&self, view: memory::View, offset: u64, it: memory::IndexBufferType) {
        let dev = self.i_pool.device();

        unsafe {
            dev.cmd_bind_index_buffer(self.i_buffer, view.buffer(), offset, it)
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

    /// Draw primitives with indexed vertices
    ///
    /// `index_count` is the number of vertices to draw
    ///
    /// `instance_count` is the number of instances to draw
    ///
    /// `first_index` is the base index within the index buffer
    ///
    /// `vertex_offset` is the value added to the vertex index before indexing into the vertex buffer
    ///
    /// `first_instance` is the instance ID of the first instance to draw
    ///
    /// See [more](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCmdDrawIndexed.html)
    pub fn draw_indexed(
        &self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        let dev = self.i_pool.device();

        unsafe {
            dev.cmd_draw_indexed(
                self.i_buffer,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
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

impl fmt::Debug for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Buffer")
        .field("i_pool", &self.i_pool)
        .field("i_buffer", &self.i_buffer)
        .finish()
    }
}

/// Buffer which is ready for execution
pub struct ExecutableBuffer {
    i_buffer: vk::CommandBuffer,
    i_pool: Pool,
}

#[doc(hidden)]
impl ExecutableBuffer {
    pub fn buffer(&self) -> &vk::CommandBuffer {
        &self.i_buffer
    }
}

impl fmt::Debug for ExecutableBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Buffer")
        .field("i_buffer", &self.i_buffer)
        .field("i_pool", &self.i_pool)
        .finish()
    }
}