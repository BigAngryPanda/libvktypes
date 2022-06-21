//! Instance extensions

use ash::extensions::ext;

use std::ffi::CStr;

pub type ExtentionNamesList<'a> = Vec<&'a CStr>;

pub const DEBUG_EXT_NAME: &CStr = ext::DebugUtils::name();