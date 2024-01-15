//! Instance extensions

use ash::extensions::{ext, khr};

use std::ffi::CStr;

use std::ffi::c_char;

use crate::window;
use raw_window_handle::HasRawDisplayHandle;

pub type ExtentionNamesList<'a> = Vec<&'a CStr>;

pub const DEBUG_EXT_NAME: *const i8 = ext::DebugUtils::name().as_ptr();

pub const SURFACE_EXT_NAME: *const i8 = khr::Surface::name().as_ptr();

pub const XLIB_SURFACE_EXT_NAME: *const i8 = khr::XlibSurface::name().as_ptr();

/// Device ext
pub const SWAPCHAIN_EXT_NAME: *const i8 = khr::Swapchain::name().as_ptr();

/// Return required extensions for surface
///
/// If function failed to do this returns `&[]`
pub fn required_extensions(window: &window::Window) -> Vec<*const c_char> {
    Vec::from(
        ash_window::enumerate_required_extensions(window.raw_display_handle()).unwrap_or(&[])
    )
}