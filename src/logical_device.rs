use ash::version::InstanceV1_0;
use ash::version::DeviceV1_0;

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
use crate::unwrap_result_or_none;

use std::ptr;

pub struct LogicalDevice {
	device: Device,
	queue: Queue,
}

impl LogicalDevice {
	pub fn new(lib: &LibHandler, desc: &HWDescription, q_family_index: usize) -> Option<LogicalDevice> {
		let priorities:[f32; 1] = [1.0_f32];

		let dev_queue_info = DeviceQueueCreateInfo {
			s_type: StructureType::DEVICE_QUEUE_CREATE_INFO,
			p_next: ptr::null(),
			flags: DeviceQueueCreateFlags::empty(),
			queue_family_index: q_family_index as u32,
			queue_count: 1,
			p_queue_priorities: &priorities as *const f32,
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

		let dev:Device = unwrap_result_or_none!(unsafe { lib.instance.create_device(desc.hw_device, &create_info, None) });

		let queue:Queue = unsafe { dev.get_device_queue(q_family_index as u32, 0) };

		let result = LogicalDevice {
			device: dev,
			queue: queue,
		};

		Some(result)
	}
}

impl Drop for LogicalDevice {
	fn drop(&mut self) {
		unsafe { self.device.destroy_device(None) };
	}
}