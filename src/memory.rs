//! Provide memory allocation functions

use ash::vk;

use std::ptr;

use crate::logical_device::LogicalDevice;
use crate::hardware::MemoryProperty;
use crate::{
	on_error,
	on_option
};

/// Ash type which representes buffer usage
///
#[doc = "Ash documentation <https://docs.rs/ash/latest/ash/vk/struct.BufferUsageFlags.html>"]
///
#[doc = "Vulkan documentation <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkBufferUsageFlagBits.html>"]
pub type BufferType = vk::BufferUsageFlags;
#[doc(hidden)]
pub type BufferDescriptor = vk::DescriptorBufferInfo;

/// Represents single memory handler
///
/// Example of memory allocation
/// ```ignore
/// // size in bytes
/// static MEMORY_SIZE:u64 = 1024;
///
/// // create logical device
/// let dev = ...;
///
/// //allocate memory
/// let test_memory = Memory::new(
///     &dev, MEMORY_SIZE,
///     MemoryProperty::HOST_VISIBLE,
///     BufferType::STORAGE_BUFFER | BufferType::TRANSFER_SRC | BufferType::TRANSFER_DST
/// );
/// ```
pub struct Memory<'a> {
	i_ldevice: &'a LogicalDevice<'a>,
	i_device_memory: vk::DeviceMemory,
	i_buffer: vk::Buffer,
	i_size: u64,
	i_flags: MemoryProperty,
}

#[derive(Debug)]
pub enum MemoryError {
	DeviceMemory,
	NoMemoryType,
	Buffer,
	MapAccess,
	Flush,
	Bind,
}

// TODO rewrite memory subsystem
// idea: memory pool return memory_type object
// by passing 'MemoryType' object we allocate actual memory (Buffer + MemoryRequirements ?)
// pros: memorize actual memory type
impl<'a> Memory<'a> {
	pub fn new(dev: &'a LogicalDevice<'a>,
			   mem_size: u64,
			   mem_props: MemoryProperty,
			   buf_type: BufferType) -> Result<Memory, MemoryError>
	{
		let buffer_info = vk::BufferCreateInfo {
			s_type: vk::StructureType::BUFFER_CREATE_INFO,
			p_next: ptr::null(),
			flags: vk::BufferCreateFlags::empty(),
			size: mem_size,
			usage: buf_type,
			sharing_mode: vk::SharingMode::EXCLUSIVE,
			queue_family_index_count: 1,
			p_queue_family_indices: &dev.i_queue_index,
		};

		let buffer: vk::Buffer = on_error!(
			unsafe { dev.i_device.create_buffer(&buffer_info, None) },
			return Err(MemoryError::Buffer)
		);

		let requirements: vk::MemoryRequirements = unsafe { dev.i_device.get_buffer_memory_requirements(buffer) };

		let mem_index: u32 = on_option!(
			dev.i_mem_info.iter().enumerate().find_map(
				|(i, d)| if ((requirements.memory_type_bits >> i) & 1) == 1 && d.is_compatible(mem_props) {
					Some(i as u32)
				}
				else {
					None
				}
			),
			return Err(MemoryError::NoMemoryType)
		);

		let memory_info = vk::MemoryAllocateInfo {
			s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
			p_next: ptr::null(),
			allocation_size: requirements.size,
			memory_type_index: mem_index,
		};

		let dev_memory:vk::DeviceMemory = on_error!(
			unsafe { dev.i_device.allocate_memory(&memory_info, None) },
			return Err(MemoryError::DeviceMemory)
		);

		// Without coherency we have to manually synchronize memory between host and device
		if !mem_props.contains(vk::MemoryPropertyFlags::HOST_COHERENT)
			&& mem_props.contains(vk::MemoryPropertyFlags::HOST_VISIBLE) {
			let mem_range = vk::MappedMemoryRange {
				s_type: vk::StructureType::MAPPED_MEMORY_RANGE,
				p_next: ptr::null(),
				memory: dev_memory,
				offset: 0,
				size: vk::WHOLE_SIZE
			};

			unsafe {
				on_error!(
					dev.i_device.map_memory(dev_memory, 0, mem_size, vk::MemoryMapFlags::empty()),
					return Err(MemoryError::MapAccess)
				);

				on_error!(
					dev.i_device.flush_mapped_memory_ranges(&[mem_range]),
					return Err(MemoryError::Flush)
				);

				dev.i_device.unmap_memory(dev_memory);
			}
		}

		on_error!(
			unsafe { dev.i_device.bind_buffer_memory(buffer, dev_memory, 0) },
			return Err(MemoryError::Bind)
		);

		Ok(
			Memory {
				i_ldevice: dev,
				i_device_memory: dev_memory,
				i_buffer: buffer,
				i_size: mem_size,
				i_flags: mem_props,
			}
		)
	}

// TODO: example?

	/// Performs action on mutable memory
	///
	/// If memory is not coherent performs
	/// [vkFlushMappedMemoryRanges](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkFlushMappedMemoryRanges.html)
	///
	/// In other words makes host memory changes available to device
	pub fn write<F>(&self, f: F) -> Result<(), MemoryError>
		where F: Fn(&mut [u8])
	{
		use core::ffi::c_void;

		let data:*mut c_void = on_error!(
			unsafe {
				self.i_ldevice.i_device.map_memory(self.i_device_memory, 0, self.i_size, vk::MemoryMapFlags::empty())
			},
			return Err(MemoryError::MapAccess)
		);

		f(unsafe {std::slice::from_raw_parts_mut(data as *mut u8, self.i_size as usize)});

		if !self.i_flags.contains(vk::MemoryPropertyFlags::HOST_COHERENT) {
			let mem_range = vk::MappedMemoryRange {
				s_type: vk::StructureType::MAPPED_MEMORY_RANGE,
				p_next: ptr::null(),
				memory: self.i_device_memory,
				offset: 0,
				size: vk::WHOLE_SIZE
			};

			on_error!(
				unsafe {
					self.i_ldevice.i_device.flush_mapped_memory_ranges(&[mem_range])
				},
				return Err(MemoryError::Flush)
			);
		}

		unsafe { self.i_ldevice.i_device.unmap_memory(self.i_device_memory) };

		Ok(())
	}

	/// Return copy of buffer's memory
	///
	/// If memory is not coherent performs
	/// [vkInvalidateMappedMemoryRanges](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkInvalidateMappedMemoryRanges.html)
	///
	/// I.e., makes device memory changes available to host (compare with [Memory::write()] method)
	///
	/// Note: on failure return same error [MemoryError::Flush]
	pub fn read(&self) -> Result<&[u8], MemoryError>
	{
		use core::ffi::c_void;

		if !self.i_flags.contains(vk::MemoryPropertyFlags::HOST_COHERENT) {
			let mem_range = vk::MappedMemoryRange {
				s_type: vk::StructureType::MAPPED_MEMORY_RANGE,
				p_next: ptr::null(),
				memory: self.i_device_memory,
				offset: 0,
				size: vk::WHOLE_SIZE
			};

			on_error!(
				unsafe {
					self.i_ldevice.i_device.invalidate_mapped_memory_ranges(&[mem_range])
				},
				return Err(MemoryError::Flush)
			);
		}

		let data:*mut c_void = on_error!(
			unsafe {
				self.i_ldevice.i_device.map_memory(self.i_device_memory, 0, self.i_size, vk::MemoryMapFlags::empty())
			},
			return Err(MemoryError::MapAccess)
		);

		let result: &[u8] = unsafe {std::slice::from_raw_parts_mut(data as *mut u8, self.i_size as usize)};

		unsafe { self.i_ldevice.i_device.unmap_memory(self.i_device_memory) };

		Ok(result)
	}

	/// Return size of the buffer in bytes
	pub fn size(&self) -> u64 {
		self.i_size
	}

	#[doc(hidden)]
	pub fn buffer(&self) -> vk::Buffer {
		self.i_buffer
	}

	#[doc(hidden)]
	pub fn get_descriptor(&self) -> BufferDescriptor {
		vk::DescriptorBufferInfo {
			buffer: self.i_buffer,
			offset: 0,
			range: vk::WHOLE_SIZE
		}
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