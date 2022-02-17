//! Provide entry point to other functions

use ash::{
	Entry,
	Instance,
};

use ash::vk::{
	ApplicationInfo,
	StructureType,
	InstanceCreateFlags,
	InstanceCreateInfo,
	DebugUtilsMessengerCreateInfoEXT,
	DebugUtilsMessageSeverityFlagsEXT,
	DebugUtilsMessageTypeFlagsEXT,
	DebugUtilsMessengerCallbackDataEXT,
	DebugUtilsMessengerCreateFlagsEXT,
	DebugUtilsMessengerEXT,
	Bool32,
};

use ash::vk::make_api_version;

use ash::vk::FALSE;

use ash::extensions::ext::DebugUtils;

use std::ptr;
use std::ffi::{
	CString,
	CStr,
	c_void,
};

use crate::on_error;

/// Entry point for entire library with specified version
///
/// Typically it will be your first object to create
pub struct LibHandler {
	pub entry: Entry,
	pub instance: Instance,
	debug_loader: DebugUtils,
	debug_messenger: DebugUtilsMessengerEXT,
}

#[derive(Debug)]
pub enum LibHandlerError {
	LibraryLoad,
	InstanceCreating,
	DebugUtilsCreating,
	Unknown
}

impl LibHandler {
	/// Request version
	///
	/// Load library at compile time (runtime is not supported for now)
	///
	/// Debug layer print information in stdout
	pub fn new(major: u32, minor: u32, patch: u32, enable_debug: bool) -> Result<LibHandler, LibHandlerError> {
		let entry:Entry = Entry::linked();

		let app_info = ApplicationInfo {
			s_type: StructureType::APPLICATION_INFO,
			p_next: ptr::null(),
			p_application_name: ptr::null(),
			application_version: 0,
			p_engine_name: ptr::null(),
			engine_version: 0,
			api_version: make_api_version(0, major, minor, patch),
		};

		let layer_names = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
        let layers: Vec<*const i8> = layer_names.iter().map(|raw_name| raw_name.as_ptr()).collect();

		let dbg_msg_info = DebugUtilsMessengerCreateInfoEXT {
	        s_type: StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
	        p_next: ptr::null(),
	        flags: DebugUtilsMessengerCreateFlagsEXT::empty(),
	        message_severity: DebugUtilsMessageSeverityFlagsEXT::WARNING |
	            // DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
	            // DebugUtilsMessageSeverityFlagsEXT::INFO |
	            DebugUtilsMessageSeverityFlagsEXT::ERROR,
	        message_type: DebugUtilsMessageTypeFlagsEXT::GENERAL
	            | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
	            | DebugUtilsMessageTypeFlagsEXT::VALIDATION,
	        pfn_user_callback: Some(LibHandler::vulkan_debug_utils_callback),
	        p_user_data: ptr::null_mut(),
	    };

	    let extension_names_raw:Vec<*const i8> = vec![DebugUtils::name().as_ptr()];

		let create_info = InstanceCreateInfo {
			s_type: StructureType::INSTANCE_CREATE_INFO,
			p_next: if enable_debug { &dbg_msg_info as *const DebugUtilsMessengerCreateInfoEXT as *const c_void } else { ptr::null() },
			flags: InstanceCreateFlags::empty(),
			p_application_info: &app_info,
			pp_enabled_layer_names: if enable_debug { layers.as_ptr() } else { ptr::null() },
	        enabled_layer_count: if enable_debug { layers.len() as u32 } else { 0 },
	        pp_enabled_extension_names: if enable_debug { extension_names_raw.as_ptr() } else { ptr::null() },
	        enabled_extension_count: if enable_debug { extension_names_raw.len() as u32 } else { 0 },
		};

		let instance:Instance = on_error!(unsafe { entry.create_instance(&create_info, None) }, 
										  return Err(LibHandlerError::InstanceCreating));

		let dbg_loader = DebugUtils::new(&entry, &instance);

		let dbg_messenger:DebugUtilsMessengerEXT = if enable_debug {
			on_error!(unsafe { dbg_loader.create_debug_utils_messenger(&dbg_msg_info, None) }, return Err(LibHandlerError::DebugUtilsCreating))
		}
		else {
			DebugUtilsMessengerEXT::null()
		};

		Ok(LibHandler {
			entry: entry,
			instance: instance,
			debug_loader: dbg_loader,
			debug_messenger: dbg_messenger,
		})
	}

	/// Provide Vulkan API entry with 1.0.0 version
	pub fn with_default() -> Result<LibHandler, LibHandlerError> {
		LibHandler::new(1, 0, 0, false)
	}

	// The callback function used in Debug Utils.
	unsafe extern "system" fn vulkan_debug_utils_callback(
	    message_severity: DebugUtilsMessageSeverityFlagsEXT,
	    message_type: DebugUtilsMessageTypeFlagsEXT,
	    p_callback_data: *const DebugUtilsMessengerCallbackDataEXT,
	    _p_user_data: *mut c_void,
	) -> Bool32 {
	    let severity = match message_severity {
	        DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
	        DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
	        DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
	        DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
	        _ => "[Unknown]",
	    };

	    let types = match message_type {
	        DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
	        DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
	        DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
	        _ => "[Unknown]",
	    };

	    let message = CStr::from_ptr((*p_callback_data).p_message);

	    println!("[Debug]{}{}{:?}", severity, types, message);

	    FALSE
	}
}

impl Drop for LibHandler {
	fn drop(&mut self) {
		if self.debug_messenger != DebugUtilsMessengerEXT::null() {
			unsafe { self.debug_loader.destroy_debug_utils_messenger(self.debug_messenger, None); }
		}

		unsafe { self.instance.destroy_instance(None); }
	}
}