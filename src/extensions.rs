//! Instance extensions

use raw_window_handle::HasDisplayHandle;

use std::ffi::CStr;

use std::ffi::c_char;

use crate::window;
use crate::on_error;

pub type ExtentionNamesList<'a> = Vec<&'a CStr>;

pub const DEBUG_EXT_NAME: *const i8 = ash::vk::EXT_DEBUG_UTILS_NAME.as_ptr();

pub const SURFACE_EXT_NAME: *const i8 = ash::vk::KHR_SURFACE_NAME.as_ptr();

pub const XLIB_SURFACE_EXT_NAME: *const i8 = ash::vk::KHR_XLIB_SURFACE_NAME.as_ptr();

/// Device ext
pub const SWAPCHAIN_EXT_NAME: *const i8 = ash::vk::KHR_SWAPCHAIN_NAME.as_ptr();

/// Return required extensions for surface
///
/// If function failed to do this returns empty vector
pub fn required_extensions(window: &window::Window) -> Vec<*const c_char> {
    let display_handle = on_error!(window.display_handle(), { return Vec::new(); });

    Vec::from(
        ash_window::enumerate_required_extensions(display_handle.as_raw()).unwrap_or(&[])
    )
}