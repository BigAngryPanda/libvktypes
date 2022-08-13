//! Provides API to the selected GPU (Logical device)
//!
//! Instead of [hw module](crate::hw) `dev` represents logical level

use ash::vk;

use crate::{libvk, hw};
use crate::on_error_ret;

use std::marker::PhantomData;
use std::ptr;

pub struct DeviceType<'a> {
    pub lib: &'a libvk::Instance,
    pub hw: &'a hw::HWDevice,
    pub queue_family_index: u32,
    pub priorities: &'a [f32],
    pub extensions: &'a [*const i8],
}

#[derive(Debug)]
pub enum DeviceError {
    Creating,
}

/// Core structure of the library
///
/// `Device` represents logical device and provide API to the selected GPU
pub struct Device<'a> {
    i_device: ash::Device,
    i_queue_index: u32,
    i_queue_count: u32,
    i_hw: &'a hw::HWDevice,
    _marker: PhantomData<&'a libvk::Instance>,
}

/// As Vulkan API specification demands instance must outlive device (and any other object which created via instance)
///
/// Hence lifetime requirements
impl<'a> Device<'a> {
    pub fn new(dev_type: &'a DeviceType) -> Result<Device<'a>, DeviceError> {
        let dev_queue_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index: dev_type.queue_family_index,
            queue_count: dev_type.priorities.len() as u32,
            p_queue_priorities: dev_type.priorities.as_ptr(),
        };

        let create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: 1,
            p_queue_create_infos: &dev_queue_info,
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(),
            enabled_extension_count: dev_type.extensions.len() as u32,
            pp_enabled_extension_names: dev_type.extensions.as_ptr(),
            p_enabled_features: ptr::null(),
        };

        let dev: ash::Device = on_error_ret!(
            unsafe { dev_type.lib.instance().create_device(dev_type.hw.device(), &create_info, None) },
            DeviceError::Creating
        );

        Ok(Device::<'a> {
            i_device: dev,
            i_queue_index: dev_type.queue_family_index,
            i_queue_count: dev_type.priorities.len() as u32,
            i_hw: dev_type.hw,
            _marker: PhantomData,
        })
    }

    #[doc(hidden)]
    pub fn queue_index(&self) -> u32 {
        self.i_queue_index
    }

    #[doc(hidden)]
    pub fn queue_count(&self) -> u32 {
        self.i_queue_count
    }

    #[doc(hidden)]
    pub fn device(&self) -> &ash::Device {
        &self.i_device
    }

    #[doc(hidden)]
    pub fn hw(&self) -> &hw::HWDevice {
        self.i_hw
    }
}

impl<'a> Drop for Device<'a> {
    fn drop(&mut self) {
        unsafe { self.i_device.destroy_device(None) };
    }
}