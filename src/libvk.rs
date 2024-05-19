//! Provide library handler
//!
//! Typically it is your first object to create

use ash;
use ash::vk;
use ash::ext::debug_utils;

use crate::on_error_ret;
use crate::layers::{DebugLayer, Layer};

use std::ptr;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct InstanceType<'a> {
    pub version_major: u32,
    pub version_minor: u32,
    pub version_patch: u32,
    pub dynamic_load: bool,
    pub debug_layer: Option<DebugLayer<'a>>,
    pub extensions: &'a [*const i8],
}

impl<'a> Default for InstanceType<'a> {
    fn default() -> InstanceType<'a> {
        InstanceType {
            version_major: 1,
            version_minor: 0,
            version_patch: 0,
            dynamic_load: false,
            debug_layer: None,
            extensions: &[],
        }
    }
}

pub struct Instance {
    i_entry: ash::Entry,
    i_instance: ash::Instance,
    i_debug_loader: debug_utils::Instance,
    i_debug_messenger: vk::DebugUtilsMessengerEXT,
}

#[derive(Debug)]
pub enum InstanceError {
    LibraryLoad,
    Instance,
    DebugUtilsCreating,
    Unknown,
}

impl Instance {
    pub fn new(desc: &InstanceType) -> Result<Instance, InstanceError> {
        let entry: ash::Entry = if desc.dynamic_load {
            on_error_ret!(unsafe { ash::Entry::load() }, InstanceError::LibraryLoad)
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
            _marker: PhantomData,
        };

        let layer_names = [DebugLayer::name()];
        let layers: Vec<*const i8> = layer_names.iter().map(|raw_name| raw_name.as_ptr()).collect();

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
                desc.extensions.as_ptr()
            },
            enabled_extension_count: if desc.extensions.is_empty() {
                0
            } else {
                desc.extensions.len() as u32
            },
            _marker: PhantomData,
        };

        let instance: ash::Instance = on_error_ret!(
            unsafe { entry.create_instance(&create_info, None) },
            InstanceError::Instance
        );

        let dbg_loader = debug_utils::Instance::new(&entry, &instance);

        let dbg_messenger: vk::DebugUtilsMessengerEXT = if let Some(layer) = &desc.debug_layer {
            on_error_ret!(unsafe { dbg_loader.create_debug_utils_messenger(layer.as_raw(), None) }, InstanceError::DebugUtilsCreating)
        }
        else {
            vk::DebugUtilsMessengerEXT::null()
        };

        Ok(Instance {
			i_entry: entry,
			i_instance: instance,
			i_debug_loader: dbg_loader,
			i_debug_messenger: dbg_messenger,
		})
    }

    #[doc(hidden)]
    pub fn instance(&self) -> &ash::Instance {
        &self.i_instance
    }

    #[doc(hidden)]
    pub fn entry(&self) -> &ash::Entry {
        &self.i_entry
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
		if self.i_debug_messenger != vk::DebugUtilsMessengerEXT::null() {
			unsafe { self.i_debug_loader.destroy_debug_utils_messenger(self.i_debug_messenger, None); }
		}

		unsafe { self.i_instance.destroy_instance(None); }
    }
}
