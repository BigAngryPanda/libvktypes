use ash::version::InstanceV1_0;

use ash::vk::{
	PhysicalDevice,
	PhysicalDeviceProperties,
	PhysicalDeviceMemoryProperties,
	MemoryType,
	MemoryPropertyFlags,
	QueueFamilyProperties,
	QueueFlags,
};

use crate::instance::LibHandler;
use crate::unwrap_result_or_none;

use std::ffi::CStr;

#[derive(Debug)]
pub struct HWDevice {
	pub hw: PhysicalDevice,
}

impl HWDevice {
	fn new(device: PhysicalDevice) -> HWDevice {
		HWDevice {
			hw: device,
		}
	}

	pub fn list(lib: &LibHandler) -> Option<Vec<HWDevice>> {
		let hw:Vec<PhysicalDevice> = unwrap_result_or_none!(unsafe { lib.instance.enumerate_physical_devices() });

		Some(hw.into_iter().map(|x| HWDevice::new(x)).collect())
	}
}

#[derive(Debug)]
pub struct QueueFamilyDescription {
	pub count: u32,
	pub support_graphics: bool,
	pub support_compute: bool,
	pub support_transfer: bool,
	pub support_sparce_binding: bool,
}

impl QueueFamilyDescription {
	fn new(property: &QueueFamilyProperties) -> QueueFamilyDescription {
		QueueFamilyDescription {
			count: property.queue_count,
			support_graphics: property.queue_flags.contains(QueueFlags::GRAPHICS),
			support_compute: property.queue_flags.contains(QueueFlags::COMPUTE),
			support_transfer: property.queue_flags.contains(QueueFlags::TRANSFER),
			support_sparce_binding: property.queue_flags.contains(QueueFlags::SPARSE_BINDING),
		}
	}

	fn from_vec(properties: Vec<QueueFamilyProperties>) -> Vec<QueueFamilyDescription> {
		properties.iter().map(|x| QueueFamilyDescription::new(x)).collect()
	}
}

#[derive(Debug)]
pub struct MemoryDescription {
	pub heap_size: u64,
	pub heap_index: u32,
	pub local: bool,
	pub host_visible: bool,
	pub host_coherent: bool,
	pub host_cached: bool,
	pub lazily_allocated: bool,
}

impl MemoryDescription {
	fn new(memory_type: &MemoryType, heap_size: u64) -> MemoryDescription {
		MemoryDescription {
			heap_size: heap_size,
			heap_index: memory_type.heap_index,
			local: memory_type.property_flags.contains(MemoryPropertyFlags::DEVICE_LOCAL),
			host_visible: memory_type.property_flags.contains(MemoryPropertyFlags::HOST_VISIBLE),
			host_coherent: memory_type.property_flags.contains(MemoryPropertyFlags::HOST_COHERENT),
			host_cached: memory_type.property_flags.contains(MemoryPropertyFlags::HOST_CACHED),
			lazily_allocated: memory_type.property_flags.contains(MemoryPropertyFlags::LAZILY_ALLOCATED),
		}
	}

	fn from_properties(properties: &PhysicalDeviceMemoryProperties) -> Vec<MemoryDescription> {
		let mut result:Vec<MemoryDescription> = Vec::new();

		for i in 0..properties.memory_type_count {
			let mem_type:&MemoryType = &properties.memory_types[i as usize];
			let heap_size:u64 = properties.memory_heaps[mem_type.heap_index as usize].size;

			result.push(MemoryDescription::new(mem_type, heap_size));
		}

		result
	}
}

#[derive(Debug)]
pub struct HWDescription {
	pub device: HWDevice,
	pub vendor_id: u32,
	pub name: String,
	pub queues: Vec<QueueFamilyDescription>,
	pub memory_info: Vec<MemoryDescription>
}

impl HWDescription {
	fn new(lib: &LibHandler, hw: PhysicalDevice) -> HWDescription {
		let properties:PhysicalDeviceProperties = unsafe { lib.instance.get_physical_device_properties(hw) };
		let queue_properties:Vec<QueueFamilyProperties> = unsafe { lib.instance.get_physical_device_queue_family_properties(hw) };

		HWDescription {
			device: HWDevice::new(hw),
			vendor_id: properties.vendor_id,
			name: unsafe { CStr::from_ptr(&properties.device_name[0]).to_str().unwrap().to_owned() },
			queues: QueueFamilyDescription::from_vec(queue_properties),
			memory_info: unsafe { MemoryDescription::from_properties(&lib.instance.get_physical_device_memory_properties(hw)) },
		}
	}

	pub fn list(lib: &LibHandler) -> Option<Vec<HWDescription>> {
		let hw:Vec<PhysicalDevice> = unwrap_result_or_none!(unsafe { lib.instance.enumerate_physical_devices() });

		Some(hw.into_iter().map(|x| HWDescription::new(lib, x)).collect())
	}
}