//! Abstraction over native surface or window object

use ash::vk;
use ash::extensions::khr;

use winit::platform::unix::WindowExtUnix;

use crate::{libvk, window, hw};
use crate::{on_error_ret, on_option};

use std::ptr;
use std::os::raw::{
    c_void,
    c_ulong,
};

pub struct SurfaceType<'a> {
    pub lib: &'a libvk::Instance,
    pub window: &'a window::Window,
}

#[derive(Debug)]
pub enum SurfaceError {
    XLibIsNotSupported,
    Creation
}

pub struct Surface {
    i_loader: khr::Surface,
    i_surface: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(surface_type: &SurfaceType) -> Result<Surface, SurfaceError> {
        let wnd = surface_type.window.window();
        let entry = surface_type.lib.entry();
        let instance = surface_type.lib.instance();

	    let x11_display: *mut c_void = on_option!(wnd.xlib_display(), return Err(SurfaceError::XLibIsNotSupported));

	    let x11_window: c_ulong = wnd.xlib_window().unwrap();

	    let x11_create_info:vk::XlibSurfaceCreateInfoKHR = vk::XlibSurfaceCreateInfoKHR {
	        s_type: vk::StructureType::XLIB_SURFACE_CREATE_INFO_KHR,
	        p_next: ptr::null(),
	        flags: vk::XlibSurfaceCreateFlagsKHR::empty(),
	        window: x11_window as vk::Window,
	        dpy: x11_display as *mut vk::Display,
	    };

	    let xlib_surface_loader = khr::XlibSurface::new(entry, instance);

        let surface_khr: vk::SurfaceKHR = on_error_ret!(
            unsafe { xlib_surface_loader.create_xlib_surface(&x11_create_info, None) }, SurfaceError::Creation
        );

        let surface_loader = khr::Surface::new(entry, instance);

        Ok(
            Surface {
                i_loader: surface_loader,
                i_surface: surface_khr,
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
/// Contains two field: [`format`](self::ImageFormat) and [`color_space`](self::ColorSpace)
///
#[doc = "Ash documentation: <https://docs.rs/ash/latest/ash/vk/struct.SurfaceFormatKHR.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkSurfaceFormatKHR.html>"]
///
/// # Example
///
/// ```
/// use libvktypes::surface::{SurfaceFormat, ImageFormat, ColorSpace};
///
/// SurfaceFormat {
///     format: ImageFormat::R8G8B8A8_UNORM,
///     color_space: ColorSpace::SRGB_NONLINEAR,
/// };
/// ```
pub type SurfaceFormat = vk::SurfaceFormatKHR;

/// Image formats
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.Format.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkFormat.html>"]
pub type ImageFormat = vk::Format;

/// Color spaces
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.ColorSpaceKHR.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkColorSpaceKHR.html>"]
pub type ColorSpace = vk::ColorSpaceKHR;

/// Present modes
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.PresentModeKHR.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkPresentModeKHR.html>"]
pub type PresentMode = vk::PresentModeKHR;

/// Image usage flags
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.ImageUsageFlags.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkImageUsageFlagBits.html>"]
pub type UsageFlags = vk::ImageUsageFlags;

/// Structure specifying a two-dimensional extent
///
/// Contains two field: `width` and `height`
///
#[doc = "Ash documentation: <https://docs.rs/ash/latest/ash/vk/struct.Extent2D.html>"]
///
#[doc = "Vulkan documentation: <https://docs.rs/ash/latest/ash/vk/struct.Extent2D.html>"]
///
/// # Example
///
/// ```
/// use libvktypes::surface::Extent2D;
///
/// Extent2D {
///     width: 1920,
///     height: 1080,
/// };
/// ```
pub type Extent2D = vk::Extent2D;

/// Structure specifying a three-dimensional extent
///
#[doc = "Ash documentation: <https://docs.rs/ash/latest/ash/vk/struct.Extent3D.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkExtent3D.html>"]
///
/// # Example
///
/// ```
/// use libvktypes::surface::Extent3D;
///
/// Extent3D {
///     width: 1920,
///     height: 1080,
///     depth: 1,
/// };
/// ```
pub type Extent3D = vk::Extent3D;

/// Value describing the transform, relative to the presentation engine’s natural orientation
///
/// It is applied to the image content prior to presentation
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.SurfaceTransformFlagsKHR.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkSurfaceTransformFlagBitsKHR.html>"]
pub type PreTransformation = vk::SurfaceTransformFlagsKHR;

/// Value indicating the alpha compositing mode to use when this surface is composited together with other surfaces on certain window systems
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.CompositeAlphaFlagsKHR.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkCompositeAlphaFlagBitsKHR.html>"]
pub type CompositeAlphaFlags = vk::CompositeAlphaFlagsKHR;

#[derive(Debug)]
pub enum CapabilitiesError {
    Modes,
    Capabilities,
    Formats
}

pub struct CapabilitiesType<'a> {
    pub hw: &'a hw::HWDevice,
    pub surface: &'a Surface
}

pub struct Capabilities {
    i_modes: Vec<vk::PresentModeKHR>,
    i_capabilities: vk::SurfaceCapabilitiesKHR,
    i_formats: Vec<vk::SurfaceFormatKHR>,
}

impl Capabilities {
    /// Query for surface capabilities for the selected hw device
    pub fn get(cap_type: &CapabilitiesType) -> Result<Capabilities, CapabilitiesError> {
        let hw = cap_type.hw;
        let surface = cap_type.surface;

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
            CapabilitiesError::Capabilities
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
    pub fn modes(&self) -> impl Iterator<Item = &PresentMode> {
        self.i_modes.iter()
    }

    /// Does surface support provided presentation mode
    pub fn is_mode_supported(&self, mode: PresentMode) -> bool {
        self.i_modes.contains(&mode)
    }

    /// Check if selected flags is supported
    pub fn is_flags_supported(&self, flags: UsageFlags) -> bool {
        self.i_capabilities.supported_usage_flags.contains(flags)
    }

    /// Return 2d extent supported by surface
    pub fn extent2d(&self) -> Extent2D {
        self.i_capabilities.current_extent
    }

    /// Return 3d extent from supported 2d extent and selected depth
    pub fn extent3d(&self, ext_depth: u32) -> Extent3D {
        Extent3D {
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
    pub fn alpha_composition(&self) -> CompositeAlphaFlags {
        self.i_capabilities.supported_composite_alpha
    }

    /// Does surface support provided alpha composition flag(s)
    pub fn is_alpha_supported(&self, alpha: CompositeAlphaFlags) -> bool {
        self.i_capabilities.supported_composite_alpha.contains(alpha)
    }

    pub fn first_alpha_composition(&self) -> Option<CompositeAlphaFlags> {
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