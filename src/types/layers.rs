//! Instance layers

use std::ffi::{c_void, CString};
use std::{
    fmt,
    ptr
};
use std::fmt::{
    Formatter,
    Debug
};

use ash::vk;

use crate::utility::debug;

pub trait Layer {
    fn info(&self) -> *const c_void;
    fn name() -> CString;
}

pub struct DebugLayer(vk::DebugUtilsMessengerCreateInfoEXT);

impl DebugLayer {
    pub fn full() -> DebugLayer {
        DebugLayer(
            vk::DebugUtilsMessengerCreateInfoEXT {
                s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
                p_next: ptr::null(),
                flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
                message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
                    vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
                    vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
                message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
                pfn_user_callback: Some(debug::vulkan_debug_utils_callback),
                p_user_data: ptr::null_mut(),
            }
        )
    }

    pub fn as_raw(&self) -> &vk::DebugUtilsMessengerCreateInfoEXT {
        &self.0
    }
}

impl Layer for DebugLayer {
    fn info(&self) -> *const c_void {
        &self.0 as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void
    }

    fn name() -> CString {
        CString::new("VK_LAYER_KHRONOS_validation").expect("Failed to create layer name")
    }
}

impl Default for DebugLayer {
    fn default() -> DebugLayer {
        DebugLayer(
            vk::DebugUtilsMessengerCreateInfoEXT {
                s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
                p_next: ptr::null(),
                flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
                message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
                    // vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
                    // vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
                message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
                pfn_user_callback: Some(debug::vulkan_debug_utils_callback),
                p_user_data: ptr::null_mut(),
            }
        )
    }
}

impl Debug for DebugLayer {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "VK_LAYER_KHRONOS_validation")
    }
}