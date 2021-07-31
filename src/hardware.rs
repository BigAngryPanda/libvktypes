use ash::version::InstanceV1_0;

use ash::vk::{
	PhysicalDevice,
	PhysicalDeviceProperties,
	PhysicalDeviceMemoryProperties,
	MemoryType,
	MemoryPropertyFlags,
	QueueFamilyProperties,
	QueueFlags,
	PhysicalDeviceType,
};

use ash::vk::{
	version_major,
	version_minor,
	version_patch,
};

use crate::instance::LibHandler;
use crate::unwrap_result_or_none;

use std::iter::Enumerate;
use std::slice::Iter;
use std::ffi::CStr;
use std::fmt;

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

impl fmt::Display for QueueFamilyDescription {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f,  "Number of queues:       {}\n\
					Support graphics:       {}\n\
					Support compute:        {}\n\
					Support transfer:       {}\n\
					Support sparce binding: {}\n", 
					self.count,
					if self.support_graphics { "yes" } else { "no" },
					if self.support_compute { "yes" } else { "no" },
					if self.support_transfer { "yes" } else { "no" },
					if self.support_sparce_binding { "yes" } else { "no" })
	}
}

// https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkPhysicalDeviceMemoryProperties.html
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

impl fmt::Display for MemoryDescription {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f,  "Heap size, bytes:        {}, in kb: {}, in mb: {}\n\
					Heap index:              {}\n\
					Memory local:            {}\n\
					Memory host visible:     {}\n\
					Memory host coherent:    {}\n\
					Memory host cached:      {}\n\
					Memory lazily allocated: {}\n",
					self.heap_size, self.heap_size / 1024, self.heap_size / (1024*1024),
					self.heap_index,
					if self.local { "yes" } else { "no" },
					if self.host_visible { "yes" } else { "no" },
					if self.host_coherent { "yes" } else { "no" },
					if self.host_cached { "yes" } else { "no" },
					if self.lazily_allocated { "yes" } else { "no" })
	}
}

// https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkPhysicalDeviceType.html
#[derive(Debug)]
pub enum HWType {
	Unknown,
	Integrated,
	Discrete,
	Virtualized,
	CPU,
}

impl HWType {
	fn new(t: PhysicalDeviceType) -> HWType {
		match t {
			PhysicalDeviceType::INTEGRATED_GPU => HWType::Integrated,
			PhysicalDeviceType::DISCRETE_GPU   => HWType::Discrete,
			PhysicalDeviceType::VIRTUAL_GPU    => HWType::Virtualized,
			PhysicalDeviceType::CPU            => HWType::CPU,
			_                                  => HWType::Unknown,
		}
	}
}

impl fmt::Display for HWType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}",
				match self {
					HWType::Unknown     => "Unknown",
					HWType::Integrated  => "Integrated GPU",
					HWType::Discrete    => "Discrete GPU",
					HWType::Virtualized => "Virtual GPU",
					HWType::CPU         => "CPU",
				} )
    }
}

#[derive(Debug)]
pub struct HWDescription {
	pub hw_device: PhysicalDevice,
	pub hw_type: HWType,
	pub hw_id: u32,
	pub version_major: u32,
	pub version_minor: u32,
	pub version_patch: u32,
	pub vendor_id: u32,
	pub name: String,
	pub queues: Vec<QueueFamilyDescription>,
	pub memory_types: Vec<MemoryDescription>
}

// khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkPhysicalDeviceProperties.html
impl HWDescription {
	fn new(lib: &LibHandler, hw: PhysicalDevice) -> HWDescription {
		let properties:PhysicalDeviceProperties = unsafe { lib.instance.get_physical_device_properties(hw) };
		let queue_properties:Vec<QueueFamilyProperties> = unsafe { 
			lib.instance.get_physical_device_queue_family_properties(hw) 
		};
		let memory_desc:Vec<MemoryDescription> = unsafe { 
			MemoryDescription::from_properties(&lib.instance.get_physical_device_memory_properties(hw)) 
		};

		HWDescription {
			hw_device: hw,
			hw_type: HWType::new(properties.device_type),
			hw_id: properties.device_id,
			version_major: version_major(properties.api_version),
			version_minor: version_minor(properties.api_version),
			version_patch: version_patch(properties.api_version),
			vendor_id: properties.vendor_id,
			name: unsafe { CStr::from_ptr(&properties.device_name[0]).to_str().unwrap().to_owned() },
			queues: QueueFamilyDescription::from_vec(queue_properties),
			memory_types: memory_desc,
		}
	}

	pub fn list(lib: &LibHandler) -> Option<Vec<HWDescription>> {
		let hw:Vec<PhysicalDevice> = unwrap_result_or_none!(unsafe { lib.instance.enumerate_physical_devices() });

		Some(hw.into_iter().map(|x| HWDescription::new(lib, x)).collect())
	}

	/*
		helper
		s - selector, function which shoud return Enumerate iterator over queues or memory_types (see get_* methods)
		p -predicate, should take QueueFamilyDescription or MemoryDescription and return bool
	*/
	pub fn find<'a, I, S, P, U>(descs: I, s: S, p: P) -> Option<(usize, usize)>
	where
		I: Iterator<Item = &'a HWDescription>,
		S: Fn(&HWDescription) -> Enumerate<std::slice::Iter<'_, U>>,
		P: Fn(&U) -> bool,
	{
		let wrapper = |(i, desc): (usize, &U)| -> Option<usize> {
			if p(desc) {
            	Some(i)
	        }
	        else {
	            None
	        }
		};

		let f = |(i, desc): (usize, &HWDescription)| -> Option<(usize, usize)> {
			match s(desc).find_map(&wrapper) {
				Some(val) => Some((i, val)),
				None => None,
			}
		};

		descs.enumerate().find_map(f)
	}

	// primarily for find
	pub fn get_queues(&self) -> Enumerate<Iter<'_, QueueFamilyDescription>> {
		self.queues.iter().enumerate()
	}

	// primarily for find
	pub fn get_memory(&self) -> Enumerate<Iter<'_, MemoryDescription>> {
		self.memory_types.iter().enumerate()
	}
}

// Call unwrap to supress warnings
impl fmt::Display for HWDescription {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f,  "Device:      {}\n\
					Device type: {}\n\
					Device id:   {}\n\
					Vendor id:   {}\n\n\
					Supported API information:\n\
					Version major: {}\n\
					Version minor: {}\n\
					Version patch: {}\n\
					Queue family information:\n\
					*****************************\n", 
					self.name,
					self.hw_type,
					self.hw_id,
					self.vendor_id,
					self.version_major,
					self.version_minor,
					self.version_patch).unwrap();

		for (i, queue) in self.queues.iter().enumerate() {
			write!(f,  "Queue family number {}\n\
						-----------------------------\n\
						{}\
						-----------------------------\n", i, queue).unwrap();
		}

		write!(f, "Memory information\n*****************************\n").unwrap();

 		for (i, info) in self.memory_types.iter().enumerate() {
			write!(f,  "Memory type {}\n\
						-----------------------------\n\
						{}\
						-----------------------------\n", i, info).unwrap();
		}

		write!(f, "#############################\n").unwrap();

		Ok(())
    }
}