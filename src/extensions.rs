//! Instance extensions

use ash::extensions::{ext, khr};

use std::ffi::CStr;

pub type ExtentionNamesList<'a> = Vec<&'a CStr>;

pub const DEBUG_EXT_NAME: &CStr = ext::DebugUtils::name();

pub const SURFACE_EXT_NAME: &CStr = khr::Surface::name();

pub const XLIB_SURFACE_EXT_NAME: &CStr = khr::XlibSurface::name();