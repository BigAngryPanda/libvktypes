//! # Provides layer on Vulkan API hardware representing types

use ash::vk;

use ash::vk::{
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
	count: u32,
	property: vk::QueueFlags,
}

impl QueueFamilyDescription {
	fn new(property: &QueueFamilyProperties) -> QueueFamilyDescription {
		QueueFamilyDescription {
			count: property.queue_count,
			property: property.queue_flags,
		}
	}

	fn from_vec(properties: Vec<QueueFamilyProperties>) -> Vec<QueueFamilyDescription> {
		properties.iter().map(|x| QueueFamilyDescription::new(x)).collect()
	}

	/// How many queues in family
	pub fn count(&self) -> u32 {
		self.count
	}

	/// Is VK_QUEUE_GRAPHICS_BIT set for queue family
	pub fn is_graphics(&self) -> bool {
		self.property.contains(QueueFlags::GRAPHICS)
	}

	/// Is VK_QUEUE_COMPUTE_BIT set for queue family
	pub fn is_compute(&self) -> bool {
		self.property.contains(QueueFlags::COMPUTE)
	}

	/// Is VK_QUEUE_TRANSFER_BIT set for queue family
	pub fn is_transfer(&self) -> bool {
		self.property.contains(QueueFlags::TRANSFER)
	}

	/// Is VK_QUEUE_SPARSE_BINDING_BIT set for queue family
	pub fn is_sparce_binding(&self) -> bool {
		self.property.contains(QueueFlags::SPARSE_BINDING)
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
					if self.is_graphics() { "yes" } else { "no" },
					if self.is_compute() { "yes" } else { "no" },
					if self.is_transfer() { "yes" } else { "no" },
					if self.is_sparce_binding() { "yes" } else { "no" })
	}
}

/// Represents information about each heap
///
#[doc = "See more <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPhysicalDeviceMemoryProperties.html>"]
#[derive(Debug, Clone)]
pub struct MemoryDescription {
	/// Heap size in bytes
	pub heap_size: u64,
	/// Corresponding heap index
	pub heap_index: u32,
	property : vk::MemoryPropertyFlags,
}

// leave only bitmask
// write methods
impl MemoryDescription {
	fn new(mem_type: &MemoryType, size: u64) -> MemoryDescription {
		MemoryDescription {
			heap_size: size,
			heap_index: mem_type.heap_index,
			property : mem_type.property_flags,
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

	/// Is VK_MEMORY_HEAP_DEVICE_LOCAL_BIT set
	pub fn is_local(&self) -> bool {
		self.property.contains(MemoryPropertyFlags::DEVICE_LOCAL)
	}

	/// Is VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT set for the memory
	pub fn is_host_visible(&self) -> bool {
		self.property.contains(MemoryPropertyFlags::HOST_VISIBLE)
	}

	/// Is VK_MEMORY_PROPERTY_HOST_COHERENT_BIT set for the memory
	pub fn is_host_coherent(&self) -> bool {
		self.property.contains(MemoryPropertyFlags::HOST_COHERENT)
	}

	/// Is VK_MEMORY_PROPERTY_HOST_CACHED_BIT set for the memory
	pub fn is_host_cached(&self) -> bool {
		self.property.contains(MemoryPropertyFlags::HOST_CACHED)
	}

	/// Is VK_MEMORY_PROPERTY_LAZILY_ALLOCATED_BIT set for the memory
	pub fn is_lazily_allocated(&self) -> bool {
		self.property.contains(MemoryPropertyFlags::LAZILY_ALLOCATED)
	}
}

impl fmt::Display for MemoryDescription {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f,  "Heap size, bytes: {}, in kb: {}, in mb: {}\n\
					Heap index:       {}\n\
					Memory properties:  \n\
					Local:            {}\n\
					Host visible:     {}\n\
					Host coherent:    {}\n\
					Host cached:      {}\n\
					Lazily allocated: {}\n",
					self.heap_size, self.heap_size / 1024, self.heap_size / (1024*1024),
					self.heap_index,
					if self.is_local() { "yes" } else { "no" },
					if self.is_host_visible() { "yes" } else { "no" },
					if self.is_host_coherent() { "yes" } else { "no" },
					if self.is_host_cached() { "yes" } else { "no" },
					if self.is_lazily_allocated() { "yes" } else { "no" })
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
	pub hw_device: vk::PhysicalDevice,
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
	fn new(lib: &LibHandler, hw: vk::PhysicalDevice) -> HWDescription {
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
		let hw:Vec<vk::PhysicalDevice> = on_error!(unsafe { lib.instance.enumerate_physical_devices() }, return None);

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