//! Array of presentable images
//!
//! See [more](https://registry.khronos.org/vulkan/specs/1.2-extensions/html/chap33.html#_wsi_swapchain)

use ash::extensions::khr;
use ash::vk;

use crate::on_error_ret;
use crate::{dev, libvk, surface};

use std::ptr;

#[derive(Debug)]
pub enum SwapchainError {
    Creating,
}

/// Swapchain configuration struct
///
/// Note:
///
/// Swapchain creation process **does not** check if `format` and `color` are supported by surface or not
///
/// It is programmer's responsibility to provide correct `format` and `color`
///
/// See [Capabilities::is_format_supported](crate::surface::Capabilities::is_format_supported)
///
/// Swapchain creation process **does not** check if `num_of_images` is valid
///
/// See [Capabilities::is_img_count_supported](crate::surface::Capabilities::is_img_count_supported)
///
/// Swapchain creation process **does not** check if `present_mode` is supported
///
/// See [Capabilities::is_mode_supported](crate::surface::Capabilities::is_mode_supported)
///
/// Swapchain creation process **does not** check if all `flags` are supported
///
/// See [Capabilities::is_flags_supported](crate::surface::Capabilities::is_flags_supported)
///
/// Swapchain creation process **does not** check if `extent` is correct
///
/// # Default
///
/// For some field you may rely on [Capabilities](crate::surface::Capabilities) methods
///
/// Such as:
///
/// [Capabilities::extent2d](crate::surface::Capabilities::extent2d) for `extent`
///
/// [Capabilities::pre_transformation](crate::surface::Capabilities::pre_transformation) for `transform`
///
/// [Capabilities::alpha_composition](crate::surface::Capabilities::alpha_composition) for `alpha`
pub struct SwapchainType<'a> {
    pub lib: &'a libvk::Instance,
    pub dev: &'a dev::Device,
    pub surface: &'a surface::Surface,
    pub num_of_images: u32,
    pub format: surface::ImageFormat,
    pub color: surface::ColorSpace,
    pub present_mode: surface::PresentMode,
    pub flags: surface::UsageFlags,
    pub extent: surface::Extent2D,
    pub transform: surface::PreTransformation,
    pub alpha: surface::CompositeAlphaFlags,
}

pub struct Swapchain {
    i_loader: khr::Swapchain,
    i_swapchain: vk::SwapchainKHR,
    i_format: vk::Format,
}

impl Swapchain {
    pub fn new(swp_type: &SwapchainType) -> Result<Swapchain, SwapchainError> {
        let loader = khr::Swapchain::new(swp_type.lib.instance(), swp_type.dev.device());

        let create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface: swp_type.surface.surface(),
            min_image_count: swp_type.num_of_images,
            image_format: swp_type.format,
            image_color_space: swp_type.color,
            image_extent: swp_type.extent,
            image_array_layers: 1,
            image_usage: swp_type.flags,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
            pre_transform: swp_type.transform,
            composite_alpha: swp_type.alpha,
            present_mode: swp_type.present_mode,
            clipped: ash::vk::TRUE,
            old_swapchain: vk::SwapchainKHR::null(),
        };

        let swapchain =
            on_error_ret!(unsafe {loader.create_swapchain(&create_info, None)}, SwapchainError::Creating);

        Ok(
            Swapchain {
                i_loader: loader,
                i_swapchain: swapchain,
                i_format: swp_type.format,
            }
        )
    }

    #[doc(hidden)]
    pub fn loader(&self) -> &khr::Swapchain {
        &self.i_loader
    }

    #[doc(hidden)]
    pub fn swapchain(&self) -> vk::SwapchainKHR {
        self.i_swapchain
    }

    #[doc(hidden)]
    pub fn format(&self) -> vk::Format {
        self.i_format
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe { self.i_loader.destroy_swapchain(self.i_swapchain, None) };
    }
}