//! Logical device type

use ash::Device;

use ash::vk::{
	DeviceQueueCreateInfo,
	DeviceQueueCreateFlags,
	StructureType,
	DeviceCreateInfo,
	DeviceCreateFlags,
	Queue,
};

use crate::instance::LibHandler;
use crate::hardware::HWDescription;
use crate::on_error;

use std::ptr;
use std::marker::PhantomData;

/// Handler to the single hardware device (or implementation)
pub struct LogicalDevice<'a> {
	#[doc(hidden)]
	pub device: Device,
	#[doc(hidden)]
	pub queue: Queue,
	_marker: PhantomData<&'a LibHandler>,
}

impl<'a> LogicalDevice<'a> {
	/// As Vulkan API specification demands instance must outlive device (and any other object which created via instance)
	///
	/// Hence lifetime requirements
	pub fn new(lib: &'a LibHandler, desc: &HWDescription, q_family_index: usize) -> Option<LogicalDevice<'a>> {
		let priorities:[f32; 1] = [1.0_f32];

		let dev_queue_info = DeviceQueueCreateInfo {
			s_type: StructureType::DEVICE_QUEUE_CREATE_INFO,
			p_next: ptr::null(),
			flags: DeviceQueueCreateFlags::empty(),
			queue_family_index: q_family_index as u32,
			queue_count: 1,
			p_queue_priorities: priorities.as_ptr(),
		};

		let create_info = DeviceCreateInfo {
			s_type: StructureType::DEVICE_CREATE_INFO,
			p_next: ptr::null(),
			flags: DeviceCreateFlags::empty(),
			queue_create_info_count: 1,
			p_queue_create_infos: &dev_queue_info,
			enabled_layer_count: 0,
			pp_enabled_layer_names: ptr::null(),
			enabled_extension_count: 0,
			pp_enabled_extension_names: ptr::null(),
			p_enabled_features: ptr::null(),
		};

		let dev:Device = on_error!(unsafe { lib.instance.create_device(desc.hw_device, &create_info, None) }, return None);

		let queue:Queue = unsafe { dev.get_device_queue(q_family_index as u32, 0) };

		let result = LogicalDevice {
			device: dev,
			queue: queue,
			_marker: PhantomData,
		};

		Some(result)
	}
}

impl<'a> Drop for LogicalDevice<'a> {
	fn drop(&mut self) {
		unsafe { self.device.destroy_device(None) };
	}
}