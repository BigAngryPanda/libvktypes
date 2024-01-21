//! Provides API to the selected GPU (Logical device)
//!
//! Instead of [hw module](crate::hw) `dev` represents logical level

use ash::vk;

use crate::{libvk, hw, alloc, queue, dev};
use crate::on_error_ret;

use std::sync::Arc;
use std::{ptr, fmt};
use std::error::Error;

/// Device configuration structure
///
/// Note: on queue creation: every queue family in [`hw`](self::DeviceCfg::hw)
/// will be enabled and every queue within family will have equal priority
pub struct DeviceCfg<'a> {
    pub lib: &'a libvk::Instance,
    pub hw: &'a hw::HWDevice,
    pub extensions: &'a [*const i8],
    pub allocator: Option<alloc::Callback>,
}

#[derive(Debug)]
pub enum DeviceError {
    Creating,
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to create Device (vkCreateDevice call failed)")
    }
}

impl Error for DeviceError {}

/// Core structure of the library
///
/// `Device` represents logical device and provide API to the selected GPU
pub struct Device {
    i_core: Arc<dev::Core>,
    i_hw: hw::HWDevice,
}

impl Device {
    /// Create new [`Device`] object according to [`DeviceCfg`]
    pub fn new(dev_type: &DeviceCfg) -> Result<Device, DeviceError> {
        let mut priorities: Vec<Vec<f32>> = Vec::new();

        let dev_queue_create_info: Vec<vk::DeviceQueueCreateInfo> = dev_type
            .hw
            .queues()
            .map(|info| {
                priorities.push(vec![1.0f32; info.count() as usize]);

                vk::DeviceQueueCreateInfo {
                    s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                    p_next: ptr::null(),
                    flags: vk::DeviceQueueCreateFlags::empty(),
                    queue_family_index: info.index(),
                    queue_count: info.count(),
                    p_queue_priorities: priorities.last().unwrap().as_ptr(),
                }
            })
            .collect();

        // Warnng: enabled_layer_count and pp_enabled_layer_names is deprecated
        #[allow(deprecated)]
        let create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: dev_queue_create_info.len() as u32,
            p_queue_create_infos: dev_queue_create_info.as_ptr(),
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(),
            enabled_extension_count: dev_type.extensions.len() as u32,
            pp_enabled_extension_names: dev_type.extensions.as_ptr(),
            p_enabled_features: dev_type.hw.features(),
        };

        let dev: ash::Device = on_error_ret!(
            unsafe { dev_type.lib.instance().create_device(dev_type.hw.device(), &create_info, None) },
            DeviceError::Creating
        );

        // Note: to prevent lifetime bounds [HWDevice](crate::hw::HWDevice) will be cloned
        //
        // It is not optimal but maybe in the future it will be fixed
        Ok(Device {
            i_core: Arc::new(dev::Core::new(dev, dev_type.allocator)),
            i_hw: dev_type.hw.clone()
        })
    }

    /// Create new queue
    ///
    /// For more information see [queue crate](crate::queue)
    pub fn get_queue(&self, cfg: &queue::QueueCfg) -> queue::Queue {
        queue::Queue::new(self, cfg)
    }

    #[doc(hidden)]
    pub fn core(&self) -> &Arc<dev::Core> {
        &self.i_core
    }

    #[doc(hidden)]
    pub fn device(&self) -> &ash::Device {
        self.i_core.device()
    }

    #[doc(hidden)]
    pub fn allocator(&self) -> Option<&alloc::Callback> {
        self.i_core.allocator()
    }

    /// Return physical device in use
    pub fn hw(&self) -> &hw::HWDevice {
        &self.i_hw
    }
}