/// Framebuffer represents a collection of images which will be used in render pass
use ash::vk;

use crate::on_error_ret;
use crate::{dev, graphics, memory};

use std::error::Error;
use std::sync::Arc;
use std::fmt;
use std::ptr;

#[derive(Debug)]
pub enum FramebufferError {
    Framebuffer,
}

impl fmt::Display for FramebufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vkCreateFramebuffer call failed")
    }
}

impl Error for FramebufferError {}

pub struct FramebufferCfg<'a, 'b : 'a> {
    pub images: &'a [memory::ImageView<'b>],
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
    pub fn new(device: &dev::Device, cfg: &FramebufferCfg) -> Result<Framebuffer, FramebufferError> {
        let img_views: Vec<vk::ImageView> = cfg.images.iter().map(|img| img.image_view()).collect();

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
        };

        let framebuffer = on_error_ret!(
            unsafe { device.device().create_framebuffer(&create_info, device.allocator()) },
            FramebufferError::Framebuffer
        );

        Ok(Framebuffer {
            i_core: device.core().clone(),
            i_frame: framebuffer,
            i_extent: cfg.extent,
        })
    }

    #[doc(hidden)]
    pub fn framebuffer(&self) -> vk::Framebuffer {
        self.i_frame
    }

    #[doc(hidden)]
    pub fn extent(&self) -> vk::Extent2D {
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