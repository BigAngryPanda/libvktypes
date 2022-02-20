//! Logical device type

use ash::Device;

use ash::vk;

use ash::vk::{
	DeviceQueueCreateInfo,
	DeviceQueueCreateFlags,
	StructureType,
	DeviceCreateInfo,
	DeviceCreateFlags,
	Queue,
};

use crate::instance::LibHandler;
use crate::hardware::{
	HWDescription,
	MemoryDescription,
	MemoryProperty,
};
use crate::memory::{
	Memory,
	MemoryAllocationError,
	BufferType,
};
use crate::{
	on_error,
	on_option,
};

use std::ptr;
use std::marker::PhantomData;

/// Handler to the single hardware device (or implementation)
pub struct LogicalDevice<'a> {
	#[doc(hidden)]
	pub i_device: Device,
	i_queue: Queue,
	i_queue_index: u32,
	i_mem_info: Vec<MemoryDescription>,
	_marker: PhantomData<&'a LibHandler>,
}

// TODO new should return Result<>
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

		let dev_queue:Queue = unsafe { dev.get_device_queue(q_family_index as u32, 0) };

		let result = LogicalDevice {
			i_device: dev,
			i_queue: dev_queue,
			i_queue_index: q_family_index as u32,
			i_mem_info: desc.memory_info.clone(),
			_marker: PhantomData,
		};

		Some(result)
	}

// TODO rewrite memory subsystem
// idea: memory pool return memory_type object
// by passing 'MemoryType' object we allocate actual memory (Buffer + MemoryRequirements ?)
// pros: memorize actual memory type

	/// Allocate memory by creating [Memory] struct
	///
	/// TODO Example
	pub fn allocate_memory(&self, mem_size: u64, mem_props: MemoryProperty, buf_type: BufferType)
		-> Result<Memory, MemoryAllocationError> {
		let buffer_info = vk::BufferCreateInfo {
			s_type: StructureType::BUFFER_CREATE_INFO,
			p_next: ptr::null(),
			flags: vk::BufferCreateFlags::empty(),
			size: mem_size,
			usage: buf_type,
			sharing_mode: vk::SharingMode::EXCLUSIVE,
			queue_family_index_count: 1,
			p_queue_family_indices: &self.i_queue_index,
		};

		let buffer:vk::Buffer = on_error!(
			unsafe { self.i_device.create_buffer(&buffer_info, None) },
			return Err(MemoryAllocationError::Buffer)
		);

		let requirements:vk::MemoryRequirements = unsafe { self.i_device.get_buffer_memory_requirements(buffer) };

		let mem_index:u32 = on_option!(
			self.i_mem_info.iter().enumerate().find_map(
				|(i, d)| if ((requirements.memory_type_bits >> i) & 1) == 1 && d.is_compatible(mem_props) {
					Some(i as u32)
				}
				else {
					None
				}
			),
			return Err(MemoryAllocationError::NoMemoryType)
		);

		let memory_info = vk::MemoryAllocateInfo {
			s_type: StructureType::MEMORY_ALLOCATE_INFO,
			p_next: ptr::null(),
			allocation_size: requirements.size,
			memory_type_index: mem_index,
		};

		let memory:vk::DeviceMemory = on_error!(
			unsafe { self.i_device.allocate_memory(&memory_info, None) },
			return Err(MemoryAllocationError::DeviceMemory)
		);

		Ok(Memory::new(self, memory, buffer, requirements.size))
	}
}

impl<'a> Drop for LogicalDevice<'a> {
	fn drop(&mut self) {
		unsafe { self.i_device.destroy_device(None) };
	}
}