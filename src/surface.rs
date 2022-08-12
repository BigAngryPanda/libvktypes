//! Represents abstraction over native surface or window object

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
}