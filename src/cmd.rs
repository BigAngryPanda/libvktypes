//! Provide API to GPU command buffers

use ash::vk;

use crate::{
    dev,
    memory,
    pipeline,
    graphics
};

use crate::on_error_ret;

use std::{ptr, cmp};
use std::iter::Iterator;
use std::sync::Arc;
use std::fmt;
use std::marker::PhantomData;

pub type ExecutableBuffer = vk::CommandBuffer;

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

#[derive(Debug)]
pub enum PoolError {
    /// Failed to
    /// [create](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateCommandPool.html)
    /// command pool
    Creating,
    /// Failed to [reset](https://docs.vulkan.org/refpages/latest/refpages/source/vkResetCommandPool.html)
    /// command pool
    Reset
}

impl std::fmt::Display for PoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Creating => write!(f, "vkCreateCommandPool call failed"),
            Self::Reset => write!(f, "vkResetCommandPool call failed")
        }
    }
}

impl std::error::Error for PoolError { }

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
                    self.i_pool, self.i_core.allocator());
        }
    }
}

/// All command buffers are allocated from `Pool`
#[derive(Debug, Clone)]
pub struct Pool(Arc<CorePool>);

impl Pool {
    pub fn new(dev: &dev::Device, queue_index: u32) -> Result<Pool, PoolError> {
        let pool_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags:  vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: queue_index,
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
    ///
    /// Note: after reset method will begin command buffer by calling
    /// [`vkBeginCommandBuffer`](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkBeginCommandBuffer.html)
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

    pub fn reset(&self, release_resources: bool) -> Result<(), PoolError> {
        let flags = if release_resources {
            vk::CommandPoolResetFlags::RELEASE_RESOURCES
        } else {
            vk::CommandPoolResetFlags::empty()
        };

        unsafe {
            match self.device().reset_command_pool(self.0.i_pool, flags) {
                Ok(_) => Ok(()),
                Err(_) => Err(PoolError::Reset)
            }
        }
    }

    pub(crate) fn device(&self) -> &ash::Device {
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
    Commit,
    /// Failed to [reset](https://docs.vulkan.org/spec/latest/chapters/cmdbuffers.html#vkResetCommandBuffer) buffer
    Reset
}

impl std::fmt::Display for BufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Creating => write!(f, "vkAllocateCommandBuffers call failed"),
            Self::Begin => write!(f, "vkBeginCommandBuffer call failed"),
            Self::Commit => write!(f, "vkBeginCommandBuffer call failed"),
            Self::Reset => write!(f, "vkResetCommandBuffer call failed")
        }
    }
}

impl std::error::Error for BufferError { }

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
    /// End buffer by calling
    /// [`vkEndCommandBuffer`](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEndCommandBuffer.html)
    ///
    /// Return executable buffer
    ///
    /// Original buffer will be available only for reset
    pub fn commit(&self) -> Result<ExecutableBuffer, BufferError> {
        let dev = self.i_pool.device();

        on_error_ret!(
            unsafe { dev.end_command_buffer(self.i_buffer) },
            BufferError::Commit
        );

        Ok(self.i_buffer)
    }

    /// Bind specifically *compute* pipeline
    ///
    /// For graphics see [`bind_graphics_pipeline`](Buffer::bind_graphics_pipeline)
    pub fn bind_compute_pipeline(
        &self,
        pipe: &pipeline::ComputePipeline,
        layout: &pipeline::PipelineLayout,
        bindings: &pipeline::PipelineBindings
    ) -> &Self {
        let dev = self.i_pool.device();

        unsafe {
            dev.cmd_bind_pipeline(
                self.i_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipe.pipeline());

            dev.cmd_bind_descriptor_sets(
                self.i_buffer,
                vk::PipelineBindPoint::COMPUTE,
                layout.layout(),
                0,
                bindings.descriptors(),
                &[]);
        }

        &self
    }

    /// Copy `src` buffer into `dst`
    ///
    /// If `dst` has less capacity then copy only first `dst.size()` bytes
    ///
    /// If `src` has less capacity then rest of the `dst` memory will be left intact
    pub fn copy_memory<T: memory::BufferView>(&self, src: T, dst: T) -> &Self {
        let dev = self.i_pool.device();

        let copy_info = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: cmp::min(src.size(), dst.size()),
        };

        unsafe {
            dev.cmd_copy_buffer(
                self.i_buffer,
                memory::get_buffer(src),
                memory::get_buffer(dst),
                &[copy_info]
            );
        }

        &self
    }

    /// Copy `src` buffer into `dst`
    ///
    /// Function does not check size of the buffers
    ///
    /// `dst` image must have one of the layouts:
    /// [`TRANSFER_DST_OPTIMAL`](memory::ImageLayout::TRANSFER_DST_OPTIMAL)
    /// [`GENERAL`](memory::ImageLayout::GENERAL)
    /// on creation or via [barrier](Buffer::set_image_barrier)
    pub fn copy_buffer_to_image<T: memory::BufferView, U: memory::ImageView>(
        &self,
        src: T,
        dst: U
    ) -> &Self {
        let dev = self.i_pool.device();

        let copy_info = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: dst.memory().layout().subresource_layer(dst.index()),
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
            image_extent: dst.extent(),
        };

        let transfer_layout = memory::ImageLayout::from_raw(
            (memory::ImageLayout::TRANSFER_DST_OPTIMAL).as_raw() |
            (memory::ImageLayout::GENERAL).as_raw()
        );

        unsafe {
            dev.cmd_copy_buffer_to_image(
                self.i_buffer,
                memory::get_buffer(src),
                memory::get_image(dst),
                transfer_layout,
                &[copy_info]);
        }

        &self
    }

    /// See [`copy_buffer_to_image`](Self::copy_buffer_to_image)
    pub fn copy_buffer_to_image_with_cfg(
        &self,
        cfg: &CopyBufferToImageCfg
    ) -> &Self {
        let dev = self.i_pool.device();

        let copy_info = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: cfg.image_subresource,
            image_offset: cfg.image_offset,
            image_extent: cfg.image_extent,
        };

        unsafe {
            dev.cmd_copy_buffer_to_image(
                self.i_buffer,
                cfg.buffer,
                cfg.image,
                cfg.dst_image_layout,
                &[copy_info]);
        }

        &self
    }

    /// Dispatch work groups
    pub fn dispatch(&self, x: u32, y: u32, z: u32) -> &Self {
        let dev = self.i_pool.device();

        unsafe {
            dev.cmd_dispatch(self.i_buffer, x, y, z);
        }

        &self
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
    pub fn set_barrier<T: memory::BufferView>(
        &self,
        mem: T,
        src_type: AccessType,
        dst_type: AccessType,
        src_stage: PipelineStage,
        dst_stage: PipelineStage,
        src_queue_family: u32,
        dst_queue_family: u32
    ) -> &Self {
        let dev = self.i_pool.device();

        let mem_barrier = vk::BufferMemoryBarrier {
            s_type: vk::StructureType::BUFFER_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask: src_type,
            dst_access_mask: dst_type,
            src_queue_family_index: src_queue_family,
            dst_queue_family_index: dst_queue_family,
            buffer: memory::get_buffer(mem),
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
                &[]);
        }

        &self
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
    pub fn set_image_barrier<T: memory::ImageView>(
        &self,
        view: T,
        src_type: AccessType,
        dst_type: AccessType,
        src_layout: memory::ImageLayout,
        dst_layout: memory::ImageLayout,
        src_stage: PipelineStage,
        dst_stage: PipelineStage,
        src_queue_family: u32,
        dst_queue_family: u32
    ) -> &Self {
        let img_barrier = vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask: src_type,
            dst_access_mask: dst_type,
            old_layout: src_layout,
            new_layout: dst_layout,
            src_queue_family_index: src_queue_family,
            dst_queue_family_index: dst_queue_family,
            image: memory::get_image(view),
            subresource_range: memory::get_subresource(view),
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
                &[img_barrier]);
        }

        &self
    }

    /// Update push constatnts with raw data
    pub fn update_push_constants(
        &self,
        layout: &pipeline::PipelineLayout,
        stage: pipeline::ShaderStage,
        data: &[u8]
    ) -> &Self {
        let dev = self.i_pool.device();

        unsafe {
            dev.cmd_push_constants(
                self.i_buffer, layout.layout(), stage, 0, data)
        }

        &self
    }

    /// Begin render pass with selected framebuffer
    ///
    /// Must be ended with [`end_render_pass`](crate::cmd::Buffer::end_render_pass)
    pub fn begin_render_pass(&self, rp: &graphics::RenderPass, fb: &memory::Framebuffer) -> &Self {
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
            dev.cmd_begin_render_pass(self.i_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
        }

        &self
    }

    /// Update vertex bindings
    ///
    /// Updating starts from **first** binding
    pub fn bind_vertex_buffers_with_offsets<T: memory::BufferView>(&self, buffers: &[(T, u64)]) -> &Self {
        let dev = self.i_pool.device();

        let vertex_buffers: Vec<vk::Buffer> = buffers.iter().map(|x| memory::get_buffer(x.0)).collect();
        let offsets: Vec<vk::DeviceSize> = buffers.iter().map(|x| x.1 as u64).collect();

        unsafe {
            dev.cmd_bind_vertex_buffers(self.i_buffer, 0, vertex_buffers.as_slice(), offsets.as_slice());
        }

        &self
    }

    pub fn bind_vertex_buffers<T: memory::BufferView>(&self, buffers: &[T]) -> &Self {
        let dev = self.i_pool.device();

        let vertex_buffers: Vec<vk::Buffer> = buffers.iter().map(|&x| memory::get_buffer(x)).collect();
        let offsets: Vec<vk::DeviceSize> = vec![0; buffers.len()];

        unsafe {
            dev.cmd_bind_vertex_buffers(self.i_buffer, 0, vertex_buffers.as_slice(), offsets.as_slice());
        }

        &self
    }

    /// Bind specifically *graphics* pipeline
    ///
    /// For graphics see [`bind_compute_pipeline`](Buffer::bind_compute_pipeline)
    pub fn bind_graphics_pipeline(&self, pipe: &pipeline::GraphicsPipeline) -> &Self {
        let dev = self.i_pool.device();

        unsafe {
            dev.cmd_bind_pipeline(self.i_buffer, vk::PipelineBindPoint::GRAPHICS, pipe.pipeline());
        }

        &self
    }

    /// Enable resource usage for the `pipeline`
    ///
    /// Each element of `offsets` must be multiple of [`hw::ubo_offset`](crate::hw::HWDevice::ubo_offset)
    ///
    /// See [more](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCmdBindDescriptorSets.html)
    ///
    /// If you do not care about `offsets` leave it as `&[]`
    pub fn bind_resources(
        &self,
        layout: &pipeline::PipelineLayout,
        res: &pipeline::PipelineBindings,
        offsets: &[u32]
    ) -> &Self {
        unsafe {
            self
            .i_pool
            .device()
            .cmd_bind_descriptor_sets(
                self.i_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                layout.layout(),
                0,
                res.descriptors(),
                offsets
            );
        }

        &self
    }

    /// Bind index buffer
    pub fn bind_index_buffer<T: memory::BufferView>(
        &self,
        view: T,
        offset: u64,
        it: memory::IndexBufferType
    ) -> &Self {
        let dev = self.i_pool.device();

        unsafe {
            dev.cmd_bind_index_buffer(self.i_buffer, memory::get_buffer(view), offset, it);
        }

        &self
    }

    /// Add `vkCmdDraw` call to the buffer
    ///
    /// About args see [more](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCmdDraw.html)
    pub fn draw(&self, vc: u32, ic: u32, fv: u32, fi: u32) -> &Self {
        let dev = self.i_pool.device();

        unsafe {
            dev.cmd_draw(self.i_buffer, vc, ic, fv, fi);
        }

        &self
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
    ) -> &Self {
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

        &self
    }

    /// Pipeline must have enabled [corresponding](pipeline::GraphicsPipelineBuilder::dynamic_scissor)
    /// dynamic state
    pub fn set_scissors_2d(&self, scissors: &[memory::Extent2D]) -> &Self {
        let dev = self.i_pool.device();

        let vk_scissors: Vec<vk::Rect2D> = scissors.iter().map(|&extent| {
            vk::Rect2D {
                offset: vk::Offset2D {
                    x: 0,
                    y: 0,
                },
                extent,
            }
        }).collect();

        unsafe {
            dev.cmd_set_scissor(self.i_buffer, 0, &vk_scissors);
        }

        self
    }

    /// Pipeline must have enabled [corresponding](pipeline::GraphicsPipelineBuilder::dynamic_viewport)
    /// dynamic state
    pub fn set_viewport(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32
    ) -> &Self {
        let dev = self.i_pool.device();

        unsafe {
            dev.cmd_set_viewport(self.i_buffer, 0, &[vk::Viewport { x, y, width, height, min_depth, max_depth }]);
        }

        self
    }

    /// Read [more](https://docs.vulkan.org/spec/latest/chapters/cmdbuffers.html#vkResetCommandBuffer)
    ///
    /// Note: after reset method will begin command buffer by calling
    /// [`vkBeginCommandBuffer`](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkBeginCommandBuffer.html)
    pub fn reset(&self, release_resources: bool) -> Result<(), BufferError> {
        let dev = self.i_pool.device();

        let flags = if release_resources {
            vk::CommandBufferResetFlags::RELEASE_RESOURCES
        } else {
            vk::CommandBufferResetFlags::empty()
        };

        unsafe {
            on_error_ret!(dev.reset_command_buffer(self.i_buffer, flags), BufferError::Reset);

            let cmd_begin_info = vk::CommandBufferBeginInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
                p_next: ptr::null(),
                flags:  vk::CommandBufferUsageFlags::empty(),
                p_inheritance_info: ptr::null(),
                _marker: PhantomData,
            };

            on_error_ret!(
                dev.begin_command_buffer(self.i_buffer, &cmd_begin_info),
                BufferError::Begin
            );
        }

        Ok(())
    }

    /// End render pass
    ///
    /// Must be after [`begin_render_pass`](crate::cmd::Buffer::begin_render_pass)
    pub fn end_render_pass(&self) -> &Self {
        let dev = self.i_pool.device();

        unsafe {
            dev.cmd_end_render_pass(self.i_buffer);
        }

        &self
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

pub struct CopyBufferToImageCfg {
    pub(crate) buffer_offset: u64,
    pub(crate) buffer_row_length: u32,
    pub(crate) buffer_image_height: u32,
    pub(crate) image_subresource: vk::ImageSubresourceLayers,
    pub(crate) image_offset: vk::Offset3D,
    pub(crate) image_extent: vk::Extent3D,
    pub(crate) buffer: vk::Buffer,
    pub(crate) image: vk::Image,
    pub(crate) dst_image_layout: vk::ImageLayout
}

impl CopyBufferToImageCfg {
    /// See [`copy_buffer_to_image`](Buffer::copy_buffer_to_image)
    ///
    /// By default all information was collected from views
    ///
    /// Use methods to override defaults
    pub fn new<T: memory::BufferView, U: memory::ImageView>(src: T, dst: U) -> CopyBufferToImageCfg {
        CopyBufferToImageCfg {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: dst.memory().layout().subresource_layer(dst.index()),
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
            image_extent: dst.extent(),
            buffer: memory::get_buffer(src),
            image: memory::get_image(dst),
            dst_image_layout: memory::ImageLayout::from_raw(
                (memory::ImageLayout::TRANSFER_DST_OPTIMAL).as_raw() |
                (memory::ImageLayout::GENERAL).as_raw())
        }
    }

    /// Default is 0
    pub fn buffer_offset(&mut self, offset: u64) -> &mut Self {
        self.buffer_offset = offset;

        self
    }

    /// Default is 0
    pub fn buffer_row_length(&mut self, length: u32) -> &mut Self {
        self.buffer_row_length = length;

        self
    }

    /// Default is 0
    pub fn buffer_image_height(&mut self, height: u32) -> &mut Self {
        self.buffer_image_height = height;

        self
    }

    pub fn image_subresource_aspect(&mut self, aspect: memory::ImageAspect) -> &mut Self {
        self.image_subresource.aspect_mask = aspect;

        self
    }

    pub fn image_subresource_mip_level(&mut self, mip_level: u32) -> &mut Self {
        self.image_subresource.mip_level = mip_level;

        self
    }

    pub fn image_subresource_base_array_layer(&mut self, base_array_layer: u32) -> &mut Self {
        self.image_subresource.base_array_layer = base_array_layer;

        self
    }

    pub fn image_subresource_layer_count(&mut self, layer_count: u32) -> &mut Self {
        self.image_subresource.layer_count = layer_count;

        self
    }

    /// Default is 0
    pub fn image_offset(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.image_offset = vk::Offset3D { x, y, z };

        self
    }

    pub fn image_extent(&mut self, width: u32, height: u32, depth: u32) -> &mut Self {
        self.image_extent = vk::Extent3D { width, height, depth };

        self
    }

    /// Default is `TRANSFER_DST_OPTIMAL | GENERAL`
    pub fn dst_image_layout(&mut self, layout: memory::ImageLayout) -> &mut Self {
        self.dst_image_layout = layout;

        self
    }
}
