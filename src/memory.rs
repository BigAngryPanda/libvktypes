//! Provide memory allocation functions

use ash::vk;

use std::ptr;

use crate::logical_device::LogicalDevice;
use crate::on_error;
/*
/// Represent buffer purpose
#[doc = "See more <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDescriptorType.html>"]
#[derive(Debug)]
pub enum BufferType {
	#[doc = "See more <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/html/vkspec.html#descriptorsets-storagebuffer>"]
	Storage,
}

impl Into<vk::DescriptorType> for &BufferType {
	fn into(self) -> vk::DescriptorType {
		match self {
			&BufferType::Storage => vk::DescriptorType::STORAGE_BUFFER,
		}
	}
}

/// Buffer type and how many we can possibly allocate from singe MemoryPool
pub type BufferInfo = (BufferType, u32);

pub struct MemoryPool<'a> {
	#[doc(hidden)]
	pub descriptor_pool: DescriptorPool,
	ldevice: &'a LogicalDevice<'a>,
	// descriptor set
	// on each allocation we update desc set
}

#[derive(Debug)]
pub enum MemoryPoolError {
	DescriptorPoolCreation,
}

impl<'a> MemoryPool<'a> {
	pub fn new(dev: &'a LogicalDevice, purpose: &[BufferInfo]) -> Result<MemoryPool<'a>, MemoryPoolError> {
		let desc_size:Vec<vk::DescriptorPoolSize> = purpose.iter().map(
			|(t, c)| vk::DescriptorPoolSize {
				ty: t.into(),
				descriptor_count: *c,
			}
		).collect();

		let pool_size:u32 = 1;//purpose.iter().map(|(_, s)| s).sum();

		let desc_info = vk::DescriptorPoolCreateInfo {
			s_type: StructureType::DESCRIPTOR_POOL_CREATE_INFO,
			p_next: ptr::null(),
			flags: vk::DescriptorPoolCreateFlags::empty(),
			max_sets: pool_size,
			pool_size_count: desc_size.len() as u32,
			p_pool_sizes: desc_size.as_ptr(),
		};

		let desc_pool = on_error!(
			unsafe {dev.device.create_descriptor_pool(&desc_info, None)},
			return Err(MemoryPoolError::DescriptorPoolCreation)
		);

		Ok(MemoryPool {
			descriptor_pool: desc_pool,
			ldevice: dev,
		})
	}
*/
/// Ash type which representes buffer usage
///
#[doc = "Ash documentation <https://docs.rs/ash/latest/ash/vk/struct.BufferUsageFlags.html>"]
///
#[doc = "Vulkan documentation <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkBufferUsageFlagBits.html>"]
pub type BufferType = vk::BufferUsageFlags;

pub struct Memory<'a> {
	i_ldevice: &'a LogicalDevice<'a>,
	i_device_memory: vk::DeviceMemory,
	i_buffer: vk::Buffer,
	i_size: u64,
}

#[derive(Debug)]
pub enum MemoryAllocationError {
	DeviceMemory,
	NoMemoryType,
	Buffer,
}

impl<'a> Memory<'a> {
	pub fn new(dev: &'a LogicalDevice<'a>,
		   dev_memory: vk::DeviceMemory,
		   buf: vk::Buffer, size: u64) -> Memory {
			Memory {
				i_ldevice: dev,
				i_device_memory: dev_memory,
				i_buffer: buf,
				i_size: size,
			}
	}

	pub fn access(&self) {
		unimplemented!()
	}
}

impl<'a> Drop for Memory<'a> {
	fn drop(&mut self) {
		unsafe {
			self.i_ldevice.i_device.destroy_buffer(self.i_buffer, None);
			self.i_ldevice.i_device.free_memory(self.i_device_memory, None);
		};
	}
}