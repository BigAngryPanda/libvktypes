//! # Provides layer on Vulkan API hardware representing types

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
	api_version_major,
	api_version_minor,
	api_version_patch,
};

use crate::instance::LibHandler;
use crate::on_error;

use std::ffi::CStr;
use std::fmt;

/// Represent information about single queue family 
///
#[doc = "See more <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/html/vkspec.html#VkQueueFlagBits>"]
#[derive(Debug)]
pub struct QueueFamilyDescription {
	/// How many queues in family
	pub count: u32,
	/// Is VK_QUEUE_GRAPHICS_BIT set for queue family
	pub support_graphics: bool,
	/// Is VK_QUEUE_COMPUTE_BIT set for queue family
	pub support_compute: bool,
	/// Is VK_QUEUE_TRANSFER_BIT set for queue family
	pub support_transfer: bool,
	/// Is VK_QUEUE_SPARSE_BINDING_BIT set for queue family
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

/// Represents information about each heap
///
#[doc = "See more <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPhysicalDeviceMemoryProperties.html>"]
#[derive(Debug)]
pub struct MemoryDescription {
	/// Heap size in bytes
	pub heap_size: u64,
	/// Describes which memory type this memory heap has
	pub memory_type: u32,
	/// Specifies that the heap corresponds to device-local memory (is VK_MEMORY_HEAP_DEVICE_LOCAL_BIT set)
	pub local: bool,
	/// Is VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT set for the memory
	pub host_visible: bool,
	/// Is VK_MEMORY_PROPERTY_HOST_COHERENT_BIT set for the memory
	pub host_coherent: bool,
	/// Is VK_MEMORY_PROPERTY_HOST_CACHED_BIT set for the memory
	pub host_cached: bool,
	/// Is VK_MEMORY_PROPERTY_LAZILY_ALLOCATED_BIT set for the memory
	pub lazily_allocated: bool,
}

impl MemoryDescription {
	fn new(mem_type: &MemoryType, heap_size: u64) -> MemoryDescription {
		MemoryDescription {
			heap_size: heap_size,
			memory_type: mem_type.heap_index,
			local: mem_type.property_flags.contains(MemoryPropertyFlags::DEVICE_LOCAL),
			host_visible: mem_type.property_flags.contains(MemoryPropertyFlags::HOST_VISIBLE),
			host_coherent: mem_type.property_flags.contains(MemoryPropertyFlags::HOST_COHERENT),
			host_cached: mem_type.property_flags.contains(MemoryPropertyFlags::HOST_CACHED),
			lazily_allocated: mem_type.property_flags.contains(MemoryPropertyFlags::LAZILY_ALLOCATED),
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
					Memory type index:       {}\n\
					Memory local:            {}\n\
					Memory host visible:     {}\n\
					Memory host coherent:    {}\n\
					Memory host cached:      {}\n\
					Memory lazily allocated: {}\n",
					self.heap_size, self.heap_size / 1024, self.heap_size / (1024*1024),
					self.memory_type,
					if self.local { "yes" } else { "no" },
					if self.host_visible { "yes" } else { "no" },
					if self.host_coherent { "yes" } else { "no" },
					if self.host_cached { "yes" } else { "no" },
					if self.lazily_allocated { "yes" } else { "no" })
	}
}

/// Represents GPU type   
///
#[doc = "See more <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPhysicalDeviceType.html>"]
#[derive(Debug, PartialEq, Eq)]
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

/// Represents information about hardware (e.g. GPU)
///
#[doc = "See more <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPhysicalDeviceProperties.html>  "]
#[doc = "About version <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/html/vkspec.html#extendingvulkan-coreversions-versionnumbers>"]
#[derive(Debug)]
pub struct HWDescription {
	/// Raw VkPhysicalDevice object
	pub hw_device: PhysicalDevice,
	/// Hardware type from VkPhysicalDeviceType
	pub hw_type: HWType,
	/// Hardware ID from VkPhysicalDeviceProperties
	pub hw_id: u32,
	/// Major API version
	pub version_major: u32,
	/// Minor API version
	pub version_minor: u32,
	/// Patch API version
	pub version_patch: u32,
	/// Vendor ID
	pub vendor_id: u32,
	/// Device name
	pub name: String,
	/// Queue family information
	pub queues: Vec<QueueFamilyDescription>,
	/// Information about each memory heap
	pub memory_info: Vec<MemoryDescription>
}

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
			version_major: api_version_major(properties.api_version),
			version_minor: api_version_minor(properties.api_version),
			version_patch: api_version_patch(properties.api_version),
			vendor_id: properties.vendor_id,
			name: unsafe { CStr::from_ptr(&properties.device_name[0]).to_str().unwrap().to_owned() },
			queues: QueueFamilyDescription::from_vec(queue_properties),
			memory_info: memory_desc,
		}
	}

	/// Return information about every available hardware
	///
	/// None on failure
	///
	/// Note: empty collection does not necessary mean failure
	pub fn list(lib: &LibHandler) -> Option<Vec<HWDescription>> {
		let hw:Vec<PhysicalDevice> = on_error!(unsafe { lib.instance.enumerate_physical_devices() }, return None);

		Some(hw.into_iter().map(|dev| HWDescription::new(lib, dev)).collect())
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

 		for (i, info) in self.memory_info.iter().enumerate() {
			write!(f,  "Memory type {}\n\
						-----------------------------\n\
						{}\
						-----------------------------\n", i, info).unwrap();
		}

		write!(f, "#############################\n").unwrap();

		Ok(())
    }
}