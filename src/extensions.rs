//! Instance extensions

use ash::extensions::{ext, khr};

use std::ffi::CStr;

pub type ExtentionNamesList<'a> = Vec<&'a CStr>;

pub const DEBUG_EXT_NAME: *const i8 = ext::DebugUtils::name().as_ptr();

pub const SURFACE_EXT_NAME: *const i8 = khr::Surface::name().as_ptr();

pub const XLIB_SURFACE_EXT_NAME: *const i8 = khr::XlibSurface::name().as_ptr();

/// Device ext
pub const SWAPCHAIN_EXT_NAME: *const i8 = khr::Swapchain::name().as_ptr();