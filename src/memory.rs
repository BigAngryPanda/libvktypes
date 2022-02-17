//! Provide memory allocation functions

use ash::vk::{
	StructureType,
	MemoryAllocateInfo,
	DeviceMemory,
	DescriptorPool,
	BufferCreateInfo,
	BufferCreateFlags,
	BufferUsageFlags,
	SharingMode,
	Buffer,
	MemoryRequirements,
};

use std::ptr;

use crate::logical_device::LogicalDevice;
use crate::on_error;

pub struct MemoryPool<'a> {
	#[doc(hidden)]
	pub descriptor_pool: DescriptorPool,
	ldevice: &'a LogicalDevice<'a>,
	// mem types info
	// descriptor set
	// on each allocation we update desc set
}

#[derive(Debug)]
pub enum MemoryPoolError {
	DescriptorPoolCreation,
}

impl<'a> MemoryPool<'a> {
	pub fn new(dev: &'a LogicalDevice) -> Result<MemoryPool<'a>, MemoryPoolError> {
		unimplemented!()
	}

/*
	In this implementation is a potential bug when we pass incompatible memory type even if there is a suitable one

	TODO fix it (by iterating over all mem types?)
*/
	pub fn allocate_memory(&self, size: u64, memory_type: u32, queues: &[u32]) -> Result<Memory, MemoryAllocationError> {
		let buffer_info = BufferCreateInfo {
			s_type: StructureType::BUFFER_CREATE_INFO,
			p_next: ptr::null(),
			flags: BufferCreateFlags::empty(),
			size: size,
			usage: BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_SRC | BufferUsageFlags::TRANSFER_DST,
			sharing_mode: SharingMode::EXCLUSIVE,
			queue_family_index_count: queues.len() as u32,
			p_queue_family_indices: queues.as_ptr(),
		};

		let buffer:Buffer = on_error!(
			unsafe { self.ldevice.device.create_buffer(&buffer_info, None) }, 
			return Err(MemoryAllocationError::Buffer)
		);

		let requirements:MemoryRequirements = unsafe { self.ldevice.device.get_buffer_memory_requirements(buffer) };

		if ((requirements.memory_type_bits >> memory_type) & 1) != 1 {
			return Err(MemoryAllocationError::UnsupportedMemoryType);
		}

		let memory_info = MemoryAllocateInfo {
			s_type: StructureType::MEMORY_ALLOCATE_INFO,
			p_next: ptr::null(),
			allocation_size: requirements.size,
			memory_type_index: memory_type,
		};

		let memory:DeviceMemory = on_error!(
			unsafe { self.ldevice.device.allocate_memory(&memory_info, None) }, 
			return Err(MemoryAllocationError::DeviceMemory)
		);

		// TODO

		let result = Memory {
			device_memory: memory,
			buffer: buffer,
			size: requirements.size,
		};

		Ok(result)
	}
}

impl<'a> Drop for MemoryPool<'a> {
	fn drop(&mut self) {
		
	}
}

#[derive(Debug)]
pub enum MemoryAllocationError {
	DeviceMemory,
	UnsupportedMemoryType,
	Buffer,
}

#[derive(Debug)]
pub struct Memory {
	pub device_memory: DeviceMemory,
	buffer: Buffer,
	size: u64,
}

impl Memory {
	pub fn access(&self) {
		unimplemented!()
	}
}