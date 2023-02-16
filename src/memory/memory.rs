use ash::vk;

use crate::{on_error, on_error_ret};
use crate::{dev, hw};

use core::ffi::c_void;
use std::error::Error;
use std::sync::Arc;
use std::fmt;
use std::ptr;

// TODO mb rewrite it with separate flags?

/// Represents buffer usage flags
///
#[doc = "Possible values: <https://docs.rs/ash/latest/ash/vk/struct.BufferUsageFlags.html>"]
///
#[doc = "Vulkan documentation: <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkBufferUsageFlagBits.html>"]
pub type BufferUsageFlags = vk::BufferUsageFlags;

/// Configuration of [`Memory`](Memory) struct
pub struct MemoryCfg<'a> {
    pub size: u64,
    pub properties: hw::MemoryProperty,
    pub usage: BufferUsageFlags,
    pub shared_access: bool,
    pub queue_families: &'a [u32]
}

/// Errors during [`Memory`](Memory) initialization and access
#[derive(Debug)]
pub enum MemoryError {
    /// Failed to [create](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateBuffer.html) buffer
    Buffer,
    /// Failed to [allocate](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkAllocateMemory.html) memory
    DeviceMemory,
    /// Failed to
    /// [map](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkMapMemory.html) memory
    MapAccess,
    /// Failed to
    /// [flush](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkFlushMappedMemoryRanges.html) memory
    Flush,
    /// Failed to
    /// [bind](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkBindBufferMemory.html) memory
    Bind
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = match self {
            MemoryError::Buffer => {
                "Failed to create buffer (vkCreateBuffer call failed)"
            },
            MemoryError::DeviceMemory => {
                "Failed to allocate memory for buffer (vkAllocateMemory call failed)"
            },
            MemoryError::MapAccess => {
                "Failed to map memory (vkMapMemory call failed)"
            },
            MemoryError::Flush => {
                "Failed to flush memory (vkFlushMappedMemoryRanges call failed)"
            },
            MemoryError::Bind => {
                "Failed to bind memory (vkBindBufferMemory call failed)"
            }
        };

        write!(f, "{:?}", err_msg)
    }
}

impl Error for MemoryError {}

/// Aligned region in memory with [specified](MemoryCfg) properties
pub struct Memory {
    i_core: Arc<dev::Core>,
    i_device_memory: vk::DeviceMemory,
    i_buffer: vk::Buffer,
    i_size: u64,
    i_flags: hw::MemoryProperty,
}

impl Memory {
    /// Allocate new region of memory
    ///
    /// Note: if memory is HOST_VISIBLE and is not HOST_COHERENT performs
    /// [map_memory](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkMapMemory.html)
    /// and
    /// [flush](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkFlushMappedMemoryRanges.html)
    /// which may result in [errors](MemoryError::MapAccess)
    pub fn allocate(device: &dev::Device, memory: &hw::MemoryDescription, mem_cfg: &MemoryCfg) -> Result<Memory, MemoryError> {
        let buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::BufferCreateFlags::empty(),
            size: mem_cfg.size,
            usage: mem_cfg.usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: mem_cfg.queue_families.len() as u32,
            p_queue_family_indices: mem_cfg.queue_families.as_ptr(),
        };

        let buffer: vk::Buffer = on_error_ret!(
            unsafe { device.device().create_buffer(&buffer_info, device.allocator()) },
            MemoryError::Buffer
        );

        let requirements: vk::MemoryRequirements = unsafe {
            device
                .device()
                .get_buffer_memory_requirements(buffer)
        };

        let memory_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: requirements.size,
            memory_type_index: memory.index(),
        };

        let dev_memory: vk::DeviceMemory = unsafe {
            on_error!(
                device.device().allocate_memory(&memory_info, device.allocator()),
                {
                    device.device().destroy_buffer(buffer, device.allocator());
                    return Err(MemoryError::DeviceMemory);
                }
            )
        };

        // Without coherency we have to manually synchronize memory between host and device
        if !mem_cfg
            .properties
            .contains(vk::MemoryPropertyFlags::HOST_COHERENT)
            && mem_cfg
                .properties
                .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
        {
            let mem_range = vk::MappedMemoryRange {
                s_type: vk::StructureType::MAPPED_MEMORY_RANGE,
                p_next: ptr::null(),
                memory: dev_memory,
                offset: 0,
                size: vk::WHOLE_SIZE,
            };

            unsafe {
                on_error!(
                    device.device().map_memory(
                        dev_memory,
                        0,
                        requirements.size,
                        vk::MemoryMapFlags::empty()
                    ),
                    {
                        device.device().free_memory(dev_memory, device.allocator());
                        device.device().destroy_buffer(buffer, device.allocator());
                        return Err(MemoryError::MapAccess);
                    }
                );

                on_error!(
                    device
                        .device()
                        .flush_mapped_memory_ranges(&[mem_range]),
                    {
                        device.device().free_memory(dev_memory, device.allocator());
                        device.device().destroy_buffer(buffer, device.allocator());
                        return Err(MemoryError::Flush);
                    }
                );

                device.device().unmap_memory(dev_memory);
            }
        }

        on_error_ret!(
            unsafe {
                device
                    .device()
                    .bind_buffer_memory(buffer, dev_memory, 0)
            },
            MemoryError::Bind
        );

        Ok(Memory {
            i_core: device.core().clone(),
            i_device_memory: dev_memory,
            i_buffer: buffer,
            i_size: mem_cfg.size,
            i_flags: mem_cfg.properties,
        })
    }

    /// Performs action on mutable memory
    ///
    /// If memory is not coherent performs
    /// [vkFlushMappedMemoryRanges](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkFlushMappedMemoryRanges.html)
    ///
    /// In other words makes host memory changes available to device
    pub fn write<T, F>(&self, f: &mut F) -> Result<(), MemoryError>
    where
        F: FnMut(&mut [T]),
    {
        let data: *mut c_void = on_error_ret!(
            unsafe {
                self.i_core.device().map_memory(
                    self.i_device_memory,
                    0,
                    self.i_size,
                    vk::MemoryMapFlags::empty(),
                )
            },
            MemoryError::MapAccess
        );

        f(unsafe { std::slice::from_raw_parts_mut(data as *mut T, (self.i_size as usize)/std::mem::size_of::<T>()) });

        if !self
            .i_flags
            .contains(vk::MemoryPropertyFlags::HOST_COHERENT)
        {
            let mem_range = vk::MappedMemoryRange {
                s_type: vk::StructureType::MAPPED_MEMORY_RANGE,
                p_next: ptr::null(),
                memory: self.i_device_memory,
                offset: 0,
                size: vk::WHOLE_SIZE,
            };

            on_error_ret!(
                unsafe {
                    self.i_core
                        .device()
                        .flush_mapped_memory_ranges(&[mem_range])
                },
                MemoryError::Flush
            );
        }

        unsafe { self.i_core.device().unmap_memory(self.i_device_memory) };

        Ok(())
    }

    /// Return copy of buffer's memory
    ///
    /// If memory is not coherent performs
    /// [vkInvalidateMappedMemoryRanges](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkInvalidateMappedMemoryRanges.html)
    ///
    /// I.e. makes device memory changes available to host (compare with [Memory::write()] method)
    ///
    /// Note: on failure return same error [MemoryError::Flush]
    pub fn read(&self) -> Result<&[u8], MemoryError> {
        if !self
            .i_flags
            .contains(vk::MemoryPropertyFlags::HOST_COHERENT)
        {
            let mem_range = vk::MappedMemoryRange {
                s_type: vk::StructureType::MAPPED_MEMORY_RANGE,
                p_next: ptr::null(),
                memory: self.i_device_memory,
                offset: 0,
                size: vk::WHOLE_SIZE,
            };

            on_error_ret!(
                unsafe {
                    self.i_core
                        .device()
                        .invalidate_mapped_memory_ranges(&[mem_range])
                },
                MemoryError::Flush
            );
        }

        let data: *mut c_void = on_error_ret!(
            unsafe {
                self.i_core.device().map_memory(
                    self.i_device_memory,
                    0,
                    self.i_size,
                    vk::MemoryMapFlags::empty(),
                )
            },
            MemoryError::MapAccess
        );

        let result: &[u8] =
            unsafe { std::slice::from_raw_parts_mut(data as *mut u8, self.i_size as usize) };

        unsafe { self.i_core.device().unmap_memory(self.i_device_memory) };

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
}

impl Drop for Memory {
    fn drop(&mut self) {
        unsafe {
            self.i_core.device().destroy_buffer(self.i_buffer, self.i_core.allocator());
            self.i_core
                .device()
                .free_memory(self.i_device_memory, self.i_core.allocator());
        };
    }
}