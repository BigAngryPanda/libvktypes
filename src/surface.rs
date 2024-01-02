//! Abstraction over native surface or window object

use ash::vk;
use ash::extensions::khr;

use raw_window_handle::HasRawDisplayHandle;
use raw_window_handle::HasRawWindowHandle;
use crate::{libvk, window, hw, memory, swapchain};
use crate::on_error_ret;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum SurfaceError {
    Create
}

impl fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = match self {
            SurfaceError::Create => {
                "Failed to create Xlib surface (vkCreateXlibSurfaceKHR call failed)"
            }
        };

        write!(f, "{:?}", err_msg)
    }
}

impl Error for SurfaceError {}

/// Note: custom allocator is not supported
pub struct Surface {
    i_loader: khr::Surface,
    i_surface: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(lib: &libvk::Instance, window: &window::Window) -> Result<Surface, SurfaceError> {
        let surface = unsafe { ash_window::create_surface(
            lib.entry(),
            lib.instance(),
            window.raw_display_handle(),
            window.raw_window_handle(),
            None,
        ).unwrap() };

        let surface_loader = khr::Surface::new(lib.entry(), lib.instance());

        Ok(
            Surface {
                i_loader: surface_loader,
                i_surface: surface,
            }
        )
    }

    #[doc(hidden)]
    pub fn loader(&self) -> &khr::Surface {
        &self.i_loader
    }

    #[doc(hidden)]
    pub fn surface(&self) -> vk::SurfaceKHR {
        self.i_surface
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { self.i_loader.destroy_surface(self.i_surface, None) };
    }
}

/// Surface formats
///
/// Contains two field: [`format`](crate::memory::ImageFormat) and [`color_space`](crate::memory::ColorSpace)
///
#[doc = "Ash documentation: <https://docs.rs/ash/latest/ash/vk/struct.SurfaceFormatKHR.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkSurfaceFormatKHR.html>"]
///
/// # Example
///
/// ```
/// use libvktypes::surface::SurfaceFormat;
/// use libvktypes::memory::{ImageFormat, ColorSpace};
///
/// SurfaceFormat {
///     format: ImageFormat::R8G8B8A8_UNORM,
///     color_space: ColorSpace::SRGB_NONLINEAR,
/// };
/// ```
pub type SurfaceFormat = vk::SurfaceFormatKHR;

/// Value describing the transform, relative to the presentation engine’s natural orientation
///
/// It is applied to the image content prior to presentation
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.SurfaceTransformFlagsKHR.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkSurfaceTransformFlagBitsKHR.html>"]
pub type PreTransformation = vk::SurfaceTransformFlagsKHR;

#[derive(Debug)]
pub enum CapabilitiesError {
    Modes,
    Surface,
    Formats
}

impl fmt::Display for CapabilitiesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = match self {
            CapabilitiesError::Modes => {
                "Failed to get present modes (vkGetPhysicalDeviceSurfacePresentModesKHR call failed)"
            },
            CapabilitiesError::Surface => {
                "Failed to get surface capabilities (vkGetPhysicalDeviceSurfaceCapabilitiesKHR call failed)"
            },
            CapabilitiesError::Formats => {
                "Failed to get surface formats (vkGetPhysicalDeviceSurfaceFormatsKHR call failed)"
            }
        };

        write!(f, "{:?}", err_msg)
    }
}

impl Error for CapabilitiesError {}

pub struct Capabilities {
    i_modes: Vec<vk::PresentModeKHR>,
    i_capabilities: vk::SurfaceCapabilitiesKHR,
    i_formats: Vec<vk::SurfaceFormatKHR>,
}

impl Capabilities {
    /// Query for surface capabilities for the selected hw device
    pub fn get(hw: &hw::HWDevice, surface: &Surface) -> Result<Capabilities, CapabilitiesError> {
        let mods = on_error_ret!(
            unsafe {
                surface.loader().get_physical_device_surface_present_modes(hw.device(), surface.surface())
            },
            CapabilitiesError::Modes
        );

        let capabilities = on_error_ret!(
            unsafe {
                surface.loader().get_physical_device_surface_capabilities(hw.device(), surface.surface())
            },
            CapabilitiesError::Surface
        );

        let formats = on_error_ret!(
            unsafe {
                surface.loader().get_physical_device_surface_formats(hw.device(), surface.surface())
            },
            CapabilitiesError::Formats
        );

        Ok(
            Capabilities {
                i_modes: mods,
                i_capabilities: capabilities,
                i_formats: formats
            }
        )
    }

    /// Return number of minimal number of images required for the swapchain
    pub fn min_img_count(&self) -> u32 {
        self.i_capabilities.min_image_count
    }

    /// Return number of max number of images supported for the swapchain
    ///
    /// Note: function return [u32::MAX] if there is no limit (max = 0) or limit is equal to [u32::MAX]
    pub fn max_img_count(&self) -> u32 {
        if self.i_capabilities.max_image_count == 0 {
            u32::MAX
        }
        else {
            self.i_capabilities.max_image_count
        }
    }

    /// Return true if `count` is in range [min_img_count; max_img_count]
    pub fn is_img_count_supported(&self, count: u32) -> bool {
        (self.min_img_count()..=self.max_img_count()).contains(&count)
    }

    /// Does surface support provided combination of format and color
    pub fn is_format_supported(&self, format: SurfaceFormat) -> bool {
        self.i_formats.contains(&format)
    }

    /// Return iterator over available surface formats and corresponding color schemes
    pub fn formats(&self) -> impl Iterator<Item = &SurfaceFormat> {
        self.i_formats.iter()
    }

    /// Return iterator over all available presentation modes
    pub fn modes(&self) -> impl Iterator<Item = &swapchain::PresentMode> {
        self.i_modes.iter()
    }

    /// Does surface support provided presentation mode
    pub fn is_mode_supported(&self, mode: swapchain::PresentMode) -> bool {
        self.i_modes.contains(&mode)
    }

    /// Check if selected flags is supported
    pub fn is_flags_supported(&self, flags: memory::UsageFlags) -> bool {
        self.i_capabilities.supported_usage_flags.contains(flags)
    }

    /// Return 2d extent supported by surface
    pub fn extent2d(&self) -> memory::Extent2D {
        self.i_capabilities.current_extent
    }

    /// Return 3d extent from supported 2d extent and selected depth
    pub fn extent3d(&self, ext_depth: u32) -> memory::Extent3D {
        memory::Extent3D {
            width: self.i_capabilities.current_extent.width,
            height: self.i_capabilities.current_extent.height,
            depth: ext_depth,
        }
    }

    /// Return current transformation
    pub fn pre_transformation(&self) -> PreTransformation {
        self.i_capabilities.current_transform
    }

    /// Retrun current composite alpha flags
    pub fn alpha_composition(&self) -> memory::CompositeAlphaFlags {
        self.i_capabilities.supported_composite_alpha
    }

    /// Does surface support provided alpha composition flag(s)
    pub fn is_alpha_supported(&self, alpha: memory::CompositeAlphaFlags) -> bool {
        self.i_capabilities.supported_composite_alpha.contains(alpha)
    }

    pub fn first_alpha_composition(&self) -> Option<memory::CompositeAlphaFlags> {
        for i in 0..4 {
            if self
                .i_capabilities
                .supported_composite_alpha
                .contains(vk::CompositeAlphaFlagsKHR::from_raw(1 << i))
            {
                return Some(vk::CompositeAlphaFlagsKHR::from_raw(1 << i));
            }
        }

        None
    }
}