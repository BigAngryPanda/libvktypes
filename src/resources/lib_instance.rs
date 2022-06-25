use ash;
use ash::vk;
use ash::extensions::ext;

use crate::on_error_ret;
use crate::types::lib_type::LibInstanceType;
use crate::types::layers::{DebugLayer, Layer};

use std::ptr;

pub struct LibInstance {
    _entry: ash::Entry,
    i_instance: ash::Instance,
    i_debug_loader: ext::DebugUtils,
    i_debug_messenger: vk::DebugUtilsMessengerEXT,
}

#[derive(Debug)]
pub enum LibInstanceError {
    LibraryLoad,
    Instance,
    DebugUtilsCreating,
    Unknown,
}

impl LibInstance {
    pub fn new(desc: &LibInstanceType) -> Result<LibInstance, LibInstanceError> {
        let entry: ash::Entry = if desc.dynamic_load {
            on_error_ret!(unsafe { ash::Entry::load() }, LibInstanceError::LibraryLoad)
        } else {
            ash::Entry::linked()
        };

        let app_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: ptr::null(),
            application_version: 0,
            p_engine_name: ptr::null(),
            engine_version: 0,
            api_version: vk::make_api_version(
                0,
                desc.version_major,
                desc.version_minor,
                desc.version_patch,
            ),
        };

        let layer_names = [DebugLayer::NAME];
        let layers: Vec<*const i8> = layer_names.iter().map(|raw_name| raw_name.as_ptr() as *const i8).collect();

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: if let Some(dbg_layer) = &desc.debug_layer {
                dbg_layer.info()
            } else {
                ptr::null()
            },
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            pp_enabled_layer_names: if desc.debug_layer.is_some() {
                layers.as_ptr()
            } else {
                ptr::null()
            },
            enabled_layer_count: if desc.debug_layer.is_some() { 1 } else { 0 },
            pp_enabled_extension_names: if desc.extensions.is_empty() {
                ptr::null()
            } else {
                &desc.extensions[0].as_ptr()
            },
            enabled_extension_count: if desc.extensions.is_empty() {
                0
            } else {
                desc.extensions.len() as u32
            },
        };

        let instance: ash::Instance = on_error_ret!(
            unsafe { entry.create_instance(&create_info, None) },
            LibInstanceError::Instance
        );

        let dbg_loader = ext::DebugUtils::new(&entry, &instance);

        let dbg_messenger: vk::DebugUtilsMessengerEXT = if let Some(layer) = &desc.debug_layer {
            on_error_ret!(unsafe { dbg_loader.create_debug_utils_messenger(layer.as_raw(), None) }, LibInstanceError::DebugUtilsCreating)
        }
        else {
            vk::DebugUtilsMessengerEXT::null()
        };

        Ok(LibInstance {
			_entry: entry,
			i_instance: instance,
			i_debug_loader: dbg_loader,
			i_debug_messenger: dbg_messenger,
		})
    }

    pub fn instance(&self) -> &ash::Instance {
        &self.i_instance
    }
}

impl Drop for LibInstance {
    fn drop(&mut self) {
		if self.i_debug_messenger != vk::DebugUtilsMessengerEXT::null() {
			unsafe { self.i_debug_loader.destroy_debug_utils_messenger(self.i_debug_messenger, None); }
		}

		unsafe { self.i_instance.destroy_instance(None); }
    }
}
