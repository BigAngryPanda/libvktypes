//! Represents memory for various purposes such as vertex buffer, uniform buffer etc.
use ash::vk;

use crate::on_error;
use crate::{dev, hw, memory, graphics};

use std::sync::Arc;
use std::ptr;
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

pub const INDEX: BufferUsageFlags = BufferUsageFlags::from_raw(
    FULL_TRANSFER.as_raw() | (BufferUsageFlags::INDEX_BUFFER).as_raw()
);

/// Size of the indices
///
#[doc = "Ash documentation about possible values <https://docs.rs/ash/latest/ash/vk/struct.IndexType.html>"]
///
#[doc = "Vulkan documentation <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkIndexType.html>"]
pub type IndexBufferType = vk::IndexType;

/// Special value for starting reassembly
pub const INDEX_REASSEMBLY_UINT32: u32 = 0xffffffff;
/// Special value for starting reassembly
pub const INDEX_REASSEMBLY_UINT16: u16 = 0xffff;
/// Special value for starting reassembly
pub const INDEX_REASSEMBLY_UINT8: u8 = 0xff;

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
    i_buffers: Vec<vk::Buffer>,
    i_subregions: Vec<memory::Subregion>,
    i_sizes: Vec<u64>,
    i_memory: memory::Region
}

impl Memory {
    pub fn allocate(
        device: &dev::Device,
        cfg: &MemoryCfg
    ) -> Result<Memory, memory::MemoryError> {
        let mut buffers: Vec<vk::Buffer> = Vec::new();
        let mut memory_requirements: Vec<vk::MemoryRequirements> = Vec::new();
        let mut sizes: Vec<u64> = Vec::new();

        for cfg in cfg.buffers {
            let sharing_mode = if cfg.simultaneous_access {
                vk::SharingMode::CONCURRENT
            } else {
                vk::SharingMode::EXCLUSIVE
            };

            let buffer_info = vk::BufferCreateInfo {
                s_type: vk::StructureType::BUFFER_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::BufferCreateFlags::empty(),
                size: cfg.size,
                usage: cfg.usage,
                sharing_mode: sharing_mode,
                queue_family_index_count: cfg.queue_families.len() as u32,
                p_queue_family_indices: cfg.queue_families.as_ptr(),
            };

            for _ in 0..cfg.count {
                sizes.push(cfg.size);

                let buffer = on_error!(unsafe {
                    device.device().create_buffer(&buffer_info, device.allocator())
                }, {
                    free_buffers(device.core(), &buffers);
                    return Err(memory::MemoryError::Buffer);
                });

                buffers.push(buffer);

                let requirements: vk::MemoryRequirements = unsafe {
                    device
                    .device()
                    .get_buffer_memory_requirements(buffer)
                };

                memory_requirements.push(requirements);
            }
        }

        let regions_info = memory::Region::calculate_subregions(device, &memory_requirements);

        let mem_desc = match memory::Region::find_memory(device.hw(), regions_info.memory_bits, cfg.properties) {
            Some(val) => val,
            None => {
                free_buffers(device.core(), &buffers);
                return Err(memory::MemoryError::NoSuitableMemory)
            },
        };

        let dev_memory = match memory::Region::allocate(device, regions_info.total_size, mem_desc) {
            Ok(val) => val,
            Err(err) => {
                free_buffers(device.core(), &buffers);
                return Err(err);
            }
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
                memory: dev_memory.memory(),
                offset: 0,
                size: vk::WHOLE_SIZE,
            };

            unsafe {
                on_error!(
                    device.device().map_memory(
                        dev_memory.memory(),
                        0,
                        dev_memory.size(),
                        vk::MemoryMapFlags::empty()
                    ),
                    {
                        free_buffers(device.core(), &buffers);
                        return Err(memory::MemoryError::MapAccess);
                    }
                );

                on_error!(
                    device
                        .device()
                        .flush_mapped_memory_ranges(&[mem_range]),
                    {
                        free_buffers(device.core(), &buffers);
                        return Err(memory::MemoryError::Flush);
                    }
                );

                device.device().unmap_memory(dev_memory.memory());
            }
        }

        for i in 0..buffers.len() {
            on_error!(
                unsafe {
                    device
                    .device()
                    .bind_buffer_memory(buffers[i], dev_memory.memory(), regions_info.subregions[i].offset)
                },
                {
                    free_buffers(device.core(), &buffers);
                    return Err(memory::MemoryError::Bind);
                }
            )
        }

        Ok(Memory {
            i_core: device.core().clone(),
            i_memory: dev_memory,
            i_buffers: buffers,
            i_sizes: sizes,
            i_subregions: regions_info.subregions
        })
    }

    /// Perfrom operation `f` over selected buffer
    ///
    /// It is relatively expensive operation as memory will be mapped and unmapped
    ///
    /// It is better to use [`map_memory`](Self::map_memory) for frequent changes
    pub fn access<T, F>(&self, f: &mut F, index: usize) -> Result<(), memory::MemoryError>
    where
        F: FnMut(&mut [T]),
    {
        self.i_memory.access(
            f,
            self.i_subregions[index].offset,
            self.i_sizes[index],
            self.i_subregions[index].allocated_size
        )
    }

    /// Return whole size of the memory in bytes
    pub fn size(&self) -> u64 {
        self.i_memory.size()
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

    /// Create [`VertexView`](crate::graphics::VertexView) for the buffer
    ///
    /// About `offset` read docs for [`VertexInputCfg`](graphics::VertexInputCfg)
    ///
    /// Buffer must contain `VERTEX_BUFFER` flag
    pub fn vertex_view(&self, index: usize, offset: u32) -> graphics::VertexView {
        graphics::VertexView::with_offset(self.view(index), offset)
    }

    /// Create and return view to the selected buffer
    pub fn view(&self, index: usize) -> memory::View {
        memory::View::new(self, index)
    }

    /// Map the whole memory into buffer
    pub fn map_memory<T>(&self) -> Result<&mut [T], memory::MemoryError> {
        self.i_memory.map_memory(0, self.i_memory.size(), self.i_memory.size())
    }

    /// Unmap the **whole** memory
    ///
    /// After this call any pointer acquired by [`Memory::map_memory`](Self::map_memory) or [`View::map_memory`](memory::View::map_memory)
    /// will be invalid
    ///
    /// You **must not** use such pointer
    pub fn unmap_memory<T>(&self) {
        self.i_memory.unmap_memory();
    }

    /// Make host memory changes visible to the device
    ///
    /// Memory **must be** HOST_VISIBLE and **must not be** HOST_COHERENT
    pub fn flush(&self) -> Result<(), memory::MemoryError> {
        self.i_memory.flush(0, self.i_memory.size())
    }

    /// Make device memory changes visible to the host
    ///
    /// Potential use cases are discussed
    /// [here](https://stackoverflow.com/questions/75324067/difference-between-vkinvalidatemappedmemoryranges-and-vkcmdpipelinebarrier-in-vu)
    pub fn sync(&self) -> Result<(), memory::MemoryError> {
        self.i_memory.sync(0, self.i_memory.size())
    }

    pub(crate) fn buffer(&self, index: usize) -> vk::Buffer {
        self.i_buffers[index]
    }

    pub(crate) fn subregions(&self) -> &Vec<memory::Subregion> {
        &self.i_subregions
    }

    pub(crate) fn sizes(&self) -> &Vec<u64> {
        &self.i_sizes
    }

    pub(crate) fn region(&self) -> &memory::Region {
        &self.i_memory
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        free_buffers(&self.i_core, &self.i_buffers);
    }
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
        .field("i_device_memory", &self.i_memory)
        .field("i_buffers", &self.i_buffers)
        .field("i_pos", &self.i_subregions)
        .finish()
    }
}

impl fmt::Display for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "core: {:?}\n\
            memory: {:?}\n",
            self.i_core,
            self.i_memory,
        ).expect("Failed to print Memory");

        for i in 0..self.i_subregions.len() {
            write!(f,
                "---------------\n\
                index: {:?}\n\
                buffer: {:?}\n\
                subregion: {:?}\n\
                size: {:?}\n",
                i,
                self.i_buffers[i],
                self.i_subregions[i],
                self.i_sizes[i]
            ).expect("Failed to print Memory");
        }

        Ok(())
    }
}