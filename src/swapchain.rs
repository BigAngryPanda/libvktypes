//! Array of presentable images
//!
//! See [more](https://registry.khronos.org/vulkan/specs/1.2-extensions/html/chap34.html#_wsi_swapchain)

use ash::khr::swapchain;
use ash::vk;

use crate::on_error_ret;
use crate::{dev, libvk, surface, sync, memory};

use std::ptr;
use std::fmt;
use std::sync::Arc;
use std::error::Error;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum SwapchainError {
    Creating,
    NextImage,
    Images
}

impl fmt::Display for SwapchainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = match self {
            SwapchainError::Creating => {
                "Failed to create swapchain (vkCreateSwapchainKHR call failed)"
            },
            SwapchainError::NextImage => {
                "Failed to create image view (vkAcquireNextImageKHR call failed)"
            },
            SwapchainError::Images => {
                "Failed to get images from swapchain"
            }
        };

        write!(f, "{:?}", err_msg)
    }
}

impl Error for SwapchainError {}

/// Present modes
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.PresentModeKHR.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkPresentModeKHR.html>"]
pub type PresentMode = vk::PresentModeKHR;

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
pub struct SwapchainCfg {
    pub num_of_images: u32,
    pub format: memory::ImageFormat,
    pub color: memory::ColorSpace,
    pub present_mode: PresentMode,
    pub flags: memory::UsageFlags,
    pub extent: memory::Extent2D,
    pub transform: surface::PreTransformation,
    pub alpha: memory::CompositeAlphaFlags,
}

pub struct Swapchain {
    i_core: Arc<dev::Core>,
    i_loader: swapchain::Device,
    i_swapchain: vk::SwapchainKHR,
    i_format: vk::Format,
    i_extent: memory::Extent2D
}

impl Swapchain {
    pub fn new(lib: &libvk::Instance,
               dev: &dev::Device,
               surface: &surface::Surface,
               swp_type: &SwapchainCfg
    ) -> Result<Swapchain, SwapchainError> {
        let loader = swapchain::Device::new(lib.instance(), dev.device());

        let create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface: surface.surface(),
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
            _marker: PhantomData,
        };

        let swapchain =
            on_error_ret!(unsafe {loader.create_swapchain(&create_info, None)}, SwapchainError::Creating);

        Ok(
            Swapchain {
                i_core: dev.core().clone(),
                i_loader: loader,
                i_swapchain: swapchain,
                i_format: swp_type.format,
                i_extent: swp_type.extent
            }
        )
    }

    pub fn next_image(&self, timeout: u64, sem: Option<&sync::Semaphore>, fence: Option<&sync::Fence>)
        -> Result<u32, SwapchainError>
    {
        let (image_index, _) = on_error_ret!(
            unsafe {
                self.i_loader.acquire_next_image(
                    self.i_swapchain,
                    timeout,
                    if let Some(s) = sem {
                        s.semaphore()
                    } else {
                        vk::Semaphore::null()
                    },
                    if let Some(f) = fence {
                        f.fence()
                    } else {
                        vk::Fence::null()
                    }
                )
            },
            SwapchainError::NextImage
        );

        Ok(image_index)
    }

    pub fn images(&self) -> Result<Vec<memory::ImageMemory>, SwapchainError> {
        let mut result: Vec<memory::ImageMemory> = Vec::new();

        let swapchain_images = on_error_ret!(
            unsafe {
                self.i_loader
                    .get_swapchain_images(self.i_swapchain)
            },
            SwapchainError::Images
        );

        for image in swapchain_images {
            let memory = on_error_ret!(
                memory::ImageMemory::preallocated(&self.i_core, image, self.i_format, self.i_extent),
                SwapchainError::Images);

            result.push(memory);
        }

        Ok(result)
    }

    #[doc(hidden)]
    pub fn loader(&self) -> &swapchain::Device {
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