//! Framebuffer represents a collection of images which will be used in render pass
//!
//! [`Framebuffer`] connects [`render pass`](graphics::render_pass::RenderPass) and [`images`](memory::image::ImageMemory)
//!
//! Detailed info you can found [here](https://stackoverflow.com/questions/39557141/what-is-the-difference-between-framebuffer-and-image-in-vulkan)
use ash::vk;

use crate::{
    dev,
    graphics,
    memory,
    on_error_ret
};

use std::sync::Arc;
use std::ptr;
use std::marker::PhantomData;

pub struct FramebufferCfg<'a, 'b : 'a, T : memory::ImageView> {
    pub images: &'a mut dyn Iterator<Item = &'b T>,
    pub extent: memory::Extent2D,
    pub render_pass: &'a graphics::RenderPass
}

pub struct Framebuffer {
    i_core: Arc<dev::Core>,
    i_frame: vk::Framebuffer,
    i_extent: vk::Extent2D
}

impl Framebuffer {
    /// Create new framebuffer from existing [image](crate::memory::ImageMemory)
    pub fn new<T : memory::ImageView>(
        device: &dev::Device,
        cfg: &mut FramebufferCfg<T>
    ) -> Result<Framebuffer, memory::MemoryError> {
        let img_views: Vec<vk::ImageView> = cfg.images.map(
            |&img| memory::get_image_view(img)
        ).collect();

        let create_info = vk::FramebufferCreateInfo {
            s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FramebufferCreateFlags::empty(),
            render_pass: cfg.render_pass.render_pass(),
            attachment_count: img_views.len() as u32,
            p_attachments: img_views.as_ptr(),
            width: cfg.extent.width,
            height: cfg.extent.height,
            layers: 1,
            _marker: PhantomData,
        };

        let framebuffer = on_error_ret!(
            unsafe { device.device().create_framebuffer(&create_info, device.allocator()) },
            memory::MemoryError::Framebuffer
        );

        Ok(Framebuffer {
            i_core: device.core().clone(),
            i_frame: framebuffer,
            i_extent: cfg.extent,
        })
    }

    pub(crate) fn framebuffer(&self) -> vk::Framebuffer {
        self.i_frame
    }

    pub(crate) fn extent(&self) -> vk::Extent2D {
        self.i_extent
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.i_core.device().destroy_framebuffer(self.i_frame, self.i_core.allocator());
        }
    }
}
