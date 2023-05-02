//! Represents memory for various purposes such as vertex buffer, uniform buffer etc.
use ash::vk;

use crate::{on_error, on_error_ret, on_option};
use crate::{dev, hw, memory, graphics};

use core::ffi::c_void;
use std::sync::Arc;
use std::ptr;
use std::error::Error;
use std::fmt;

/// Purpose of buffer
///
#[doc = "Ash documentation about possible values <https://docs.rs/ash/latest/ash/vk/struct.BufferUsageFlags.html>"]
///
#[doc = "Vulkan documentation <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkBufferUsageFlagBits.html>"]
pub type BufferUsageFlags = vk::BufferUsageFlags;

// Workaround
pub const FULL_TRANSFER: BufferUsageFlags = BufferUsageFlags::from_raw(
    (BufferUsageFlags::TRANSFER_SRC).as_raw() | (BufferUsageFlags::TRANSFER_DST).as_raw()
);

pub const STORAGE: BufferUsageFlags = BufferUsageFlags::from_raw(
    FULL_TRANSFER.as_raw() | (BufferUsageFlags::STORAGE_BUFFER).as_raw()
);

pub const UNIFORM: BufferUsageFlags = BufferUsageFlags::from_raw(
    FULL_TRANSFER.as_raw() | (BufferUsageFlags::UNIFORM_BUFFER).as_raw()
);

pub const VERTEX: BufferUsageFlags = BufferUsageFlags::from_raw(
    FULL_TRANSFER.as_raw() | (BufferUsageFlags::VERTEX_BUFFER).as_raw()
);

/// Errors during memory allocation, initialization and access
#[derive(Debug)]
pub enum MemoryError {
    /// Failed to [create](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateBuffer.html) buffer
    Buffer,
    /// Failed to find suitable memory
    NoSuitableMemory,
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
            MemoryError::NoSuitableMemory => {
                "Failed to find suitable memory"
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

/// Configuration struct for memory region
#[derive(Debug, Clone)]
pub struct BufferCfg<'a> {
    pub size: u64,
    pub usage: BufferUsageFlags,
    pub queue_families: &'a [u32],
    /// Will two or more queues have access to the buffer at the same time
    pub simultaneous_access: bool,
    /// How many of this buffer you want to allocate one by one
    ///
    /// For example
    /// `[<buffer cfg, count == 1>, <buffer cfg, count == 1>]` is equivalent to `[<buffer cfg, count == 2>]`
    ///
    /// Hence each buffer will be handled separately (e.g. for alignment)
    pub count: usize
}

/// Configuration struct for memory
#[derive(Clone)]
pub struct MemoryCfg<'a, 'b : 'a> {
    pub properties: hw::MemoryProperty,
    pub filter: &'a dyn Fn(&hw::MemoryDescription) -> bool,
    pub buffers: &'a [&'a BufferCfg<'b>]
}

/// Aligned region of memory
///
/// # Allocation
/// Memory allocated in single chunk
/// in order which is provided by [`MemoryCfg`]
/// so no rearranges will be performed
///
/// Size of allocated memory is greater or equal to the requested size
/// (sum of all [`BufferCfg::size`] in [`MemoryCfg::buffers`]) due to alignment requirements
///
/// # Alignment
/// Each buffer from [`MemoryCfg::buffers`] will be separately aligned at least
/// for [`hw::memory_alignment`](crate::hw::HWDevice::memory_alignment)
///
/// Various types of buffers (uniform, storage etc.) may have their own alignment requirements
/// such as [`hw::ub_offset`](crate::hw::HWDevice::ub_offset) or [`hw::storage_offset`](crate::hw::HWDevice::storage_offset)
///
/// Hint: you may print struct (as [`Memory`] implements [`fmt::Display`]) to see memory layout
///
/// # Memory View
/// Whole memory chunk is split into regions (buffers) which are defined by [`MemoryCfg::buffers`]
///
/// To help with managing regions [`Memory View`](crate::memory::View) struct was provided
pub struct Memory {
    i_core: Arc<dev::Core>,
    i_device_memory: vk::DeviceMemory,
    i_buffers: Vec<vk::Buffer>,
    i_pos: Vec<(u64, u64, u64)>, // (offset, size, allocated size)
    i_size: u64,
    i_flags: hw::MemoryProperty
}

impl Memory {
    pub fn allocate(
        device: &dev::Device,
        cfg: &MemoryCfg
    ) -> Result<Memory, MemoryError> {
        let mut buffers: Vec<vk::Buffer> = Vec::new();
        let mut memory_type_bits = 0xffffffffu32;
        let mut last = 0u64;
        let mut pos: Vec<(u64, u64, u64)> = Vec::new();
        let mut total_size = 0u64;

        for cfg in cfg.buffers {
            let buffer_info = vk::BufferCreateInfo {
                s_type: vk::StructureType::BUFFER_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::BufferCreateFlags::empty(),
                size: cfg.size,
                usage: cfg.usage,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                queue_family_index_count: cfg.queue_families.len() as u32,
                p_queue_family_indices: cfg.queue_families.as_ptr(),
            };

            for _ in 0..cfg.count {
                if let Ok(buffer) = unsafe {
                    device.device().create_buffer(&buffer_info, device.allocator())
                } {
                    buffers.push(buffer);

                    let requirements: vk::MemoryRequirements = unsafe {
                        device
                        .device()
                        .get_buffer_memory_requirements(buffer)
                    };

                    // On one hand memory should be aligned for nonCoherentAtomSize
                    // On the other side for requirements.alignment
                    // So resulting alignment will be hcf(nonCoherentAtomSize, requirements.alignment)
                    // Spec states that both of them are power of two so calculation may be reduced
                    // To calculating max of the values
                    // See https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#limits
                    // https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#VkMemoryRequirements
                    // https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkMemoryRequirements.html
                    let alignment = std::cmp::max(device.hw().memory_alignment(), requirements.alignment);

                    // How many bytes we need after *previous* buffer
                    let begin_offset = offset(last, alignment);

                    // How many bytes we need after *current* buffer
                    let end_offset = offset(requirements.size, alignment);

                    let aligned_size = requirements.size + end_offset;

                    last += begin_offset;
                    pos.push((last, cfg.size, aligned_size));

                    memory_type_bits &= requirements.memory_type_bits;

                    last += aligned_size;

                    total_size += requirements.size + alignment;
                } else {
                    free_buffers(device.core(), &buffers);
                    return Err(MemoryError::Buffer);
                }
            }
        }

        let filter = |desc: &hw::MemoryDescription| -> bool {
            (cfg.filter)(desc)
            && ((memory_type_bits >> desc.index()) & 1) == 1
            && desc.is_compatible(cfg.properties)
            && desc.heap_size() >= total_size
        };

        let memory = on_option!(
            device.hw().find_first_memory(filter),
            {
                free_buffers(device.core(), &buffers);
                return Err(MemoryError::Buffer);
            }
        );

        let memory_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: total_size,
            memory_type_index: memory.index(),
        };

        let dev_memory: vk::DeviceMemory = unsafe {
            on_error!(
                device.device().allocate_memory(&memory_info, device.allocator()),
                {
                    free_buffers(device.core(), &buffers);
                    return Err(MemoryError::DeviceMemory);
                }
            )
        };

        // Without coherency we have to manually synchronize memory between host and device
        if !cfg
            .properties
            .contains(vk::MemoryPropertyFlags::HOST_COHERENT)
            && cfg
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
                        total_size,
                        vk::MemoryMapFlags::empty()
                    ),
                    {
                        device.device().free_memory(dev_memory, device.allocator());
                        free_buffers(device.core(), &buffers);
                        return Err(MemoryError::MapAccess);
                    }
                );

                on_error!(
                    device
                        .device()
                        .flush_mapped_memory_ranges(&[mem_range]),
                    {
                        device.device().free_memory(dev_memory, device.allocator());
                        free_buffers(device.core(), &buffers);
                        return Err(MemoryError::Flush);
                    }
                );

                device.device().unmap_memory(dev_memory);
            }
        }

        for i in 0..buffers.len() {
            on_error!(
                unsafe {
                    device
                    .device()
                    .bind_buffer_memory(buffers[i], dev_memory, pos[i].0)
                },
                {
                    unsafe {
                        device.device().free_memory(dev_memory, device.allocator())
                    };
                    free_buffers(device.core(), &buffers);
                    return Err(MemoryError::Bind);
                }
            )
        }

        Ok(Memory {
            i_core: device.core().clone(),
            i_device_memory: dev_memory,
            i_buffers: buffers,
            i_pos: pos,
            i_size: total_size,
            i_flags: cfg.properties
        })
    }

    pub fn access<T, F>(&self, f: &mut F, index: usize) -> Result<(), memory::MemoryError>
    where
        F: FnMut(&mut [T]),
    {
        let data: *mut c_void = on_error_ret!(
            unsafe {
                self.i_core.device().map_memory(
                    self.i_device_memory,
                    self.i_pos[index].0,
                    self.i_pos[index].2,
                    vk::MemoryMapFlags::empty(),
                )
            },
            memory::MemoryError::MapAccess
        );

        f(unsafe { std::slice::from_raw_parts_mut(data as *mut T, (self.i_pos[index].1 as usize)/std::mem::size_of::<T>()) });

        if !self
            .i_flags
            .contains(vk::MemoryPropertyFlags::HOST_COHERENT)
        {
            let mem_range = vk::MappedMemoryRange {
                s_type: vk::StructureType::MAPPED_MEMORY_RANGE,
                p_next: ptr::null(),
                memory: self.i_device_memory,
                offset: self.i_pos[index].0,
                size: self.i_pos[index].2,
            };

            on_error_ret!(
                unsafe {
                    self.i_core
                        .device()
                        .flush_mapped_memory_ranges(&[mem_range])
                },
                memory::MemoryError::Flush
            );
        }

        unsafe { self.i_core.device().unmap_memory(self.i_device_memory) };

        Ok(())
    }

    /// Return whole size of the memory in bytes
    pub fn size(&self) -> u64 {
        self.i_size
    }

    /// Create and return views to the buffers
    pub fn views(&self) -> Vec<memory::View> {
        self
        .i_buffers
        .iter()
        .enumerate()
        .map(|(i, _)| memory::View::new(self, i))
        .collect()
    }

    /// Create [`resource`](crate::graphics::Resource) handler from selected buffer
    pub fn resource(&self,
        index: usize,
        resource_type: graphics::ResourceType,
        stage: graphics::ShaderStage
    ) -> graphics::Resource {
        graphics::Resource::new(&[self.view(index)], resource_type, stage)
    }

    /// Return offset for the selected buffer
    pub fn buffer_offset(&self, index: usize) -> u64 {
        self.i_pos[index].0
    }

    /// Return size for the selected buffer
    pub fn buffer_size(&self, index: usize) -> u64 {
        self.i_pos[index].1
    }

    /// Return size of the buffer with respect to the alignment
    pub fn buffer_allocated_size(&self, index: usize) -> u64 {
        self.i_pos[index].2
    }

    /// Create and return view to the selected buffer
    pub fn view(&self, index: usize) -> memory::View {
        memory::View::new(self, index)
    }

    #[doc(hidden)]
    pub(crate) fn buffer(&self, index: usize) -> vk::Buffer {
        self.i_buffers[index]
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        unsafe {
            free_buffers(&self.i_core, &self.i_buffers);
            self.i_core
                .device()
                .free_memory(self.i_device_memory, self.i_core.allocator());
        };
    }
}

#[inline]
fn offset(last: u64, alignment: u64) -> u64 {
    ((last % alignment != 0) as u64)*(alignment - last % alignment)
}

fn free_buffers(device: &dev::Core, buffers: &Vec<vk::Buffer>) {
    for &buffer in buffers {
        unsafe {
            device.device().destroy_buffer(buffer, device.allocator());
        }
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Memory")
        .field("i_core", &self.i_core)
        .field("i_device_memory", &(&self.i_device_memory as *const vk::DeviceMemory))
        .field("i_buffers", &self.i_buffers)
        .field("i_pos", &self.i_pos)
        .field("i_size", &self.i_size)
        .field("i_flags", &self.i_flags)
        .finish()
    }
}

impl fmt::Display for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "core: {:?}\n\
            memory: {:?}\n\
            id: {:?}\n\
            size: {:?} ({:#x})\n\
            flags: {:#?}\n",
            self.i_core,
            &(&self.i_device_memory as *const vk::DeviceMemory),
            self.i_device_memory,
            self.i_size, self.i_size,
            self.i_flags
        ).expect("Failed to print Memory");

        for i in 0..self.i_pos.len() {
            write!(f,
                "---------------\n\
                buffer {:?}\n\
                id: {:?}\n\
                offset: {:?} ({:#x})\n\
                size: {:?} ({:#x})\n\
                allocated size: {:?} ({:#x})\n",
                i,
                self.i_buffers[i],
                self.i_pos[i].0, self.i_pos[i].0,
                self.i_pos[i].1, self.i_pos[i].1,
                self.i_pos[i].2, self.i_pos[i].2
            ).expect("Failed to print Memory");
        }

        Ok(())
    }
}