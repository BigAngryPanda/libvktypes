use ash::vk;

use core::ffi::c_void;
use std::sync::Arc;
use std::fmt;

use crate::{on_error, on_error_ret};
use crate::{dev, hw, memory};

use std::ptr;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Subregion {
    pub offset: u64,
    pub allocated_size: u64
}

impl fmt::Display for Subregion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "offset: {:?} ({:#x})\n\
            allocated size: {:?} ({:#x})\n",
            self.offset, self.offset,
            self.allocated_size, self.allocated_size
        ).expect("Failed to print Subregion");

        Ok(())
    }
}

impl Subregion {
    fn new(offset: u64, allocated_size: u64) -> Subregion {
        Subregion {
            offset: offset,
            allocated_size: allocated_size
        }
    }
}

pub(crate) struct RegionInfo {
    pub subregions: Vec<Subregion>,
    pub total_size: u64,
    pub memory_bits: u32
}

pub(crate) struct Region {
    i_core: Arc<dev::Core>,
    i_memory: vk::DeviceMemory,
    i_size: u64,
    i_flags: hw::MemoryProperty
}

impl Region {
    pub(crate) fn calculate_subregions(
        device: &dev::Device,
        requirements: &[vk::MemoryRequirements]) -> RegionInfo
    {
        let mut memory_type_bits = 0xffffffffu32;
        let mut last = 0u64;
        let mut total_size = 0u64;
        let mut pos: Vec<Subregion> = Vec::new();

        for requirement in requirements {
            // On one hand memory should be aligned for nonCoherentAtomSize
            // On the other side for requirements.alignment
            // So resulting alignment will be hcf(nonCoherentAtomSize, requirements.alignment)
            // Spec states that both of them are power of two so calculation may be reduced
            // To calculating max of the values
            // See https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#limits
            // https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#VkMemoryRequirements
            // https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkMemoryRequirements.html
            let alignment = std::cmp::max(device.hw().memory_alignment(), requirement.alignment);

            // How many bytes we need after *previous* buffer
            let begin_offset = offset(last, alignment);

            // How many bytes we need after *current* buffer
            let end_offset = offset(requirement.size, alignment);

            let aligned_size = requirement.size + end_offset;

            last += begin_offset;
            pos.push(Subregion::new(last, requirement.size));

            memory_type_bits &= requirement.memory_type_bits;

            last += aligned_size;

            total_size += requirement.size + alignment;
        }

        RegionInfo {
            subregions: pos,
            total_size: total_size,
            memory_bits: memory_type_bits
        }
    }

    pub(crate) fn allocate(device: &dev::Device, size: u64, desc: &hw::MemoryDescription) -> Result<Region, memory::MemoryError> {
        let memory_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: size,
            memory_type_index: desc.index(),
        };

        let dev_memory: vk::DeviceMemory = unsafe {
            on_error_ret!(
                device.device().allocate_memory(&memory_info, device.allocator()),
                memory::MemoryError::DeviceMemory
            )
        };

        // Without coherency we have to manually synchronize memory between host and device
        if !desc.is_compatible(vk::MemoryPropertyFlags::HOST_COHERENT)
            && desc.is_compatible(vk::MemoryPropertyFlags::HOST_VISIBLE)
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
                        size,
                        vk::MemoryMapFlags::empty()
                    ),
                    {
                        device.device().free_memory(dev_memory, device.allocator());
                        return Err(memory::MemoryError::MapAccess);
                    }
                );

                on_error!(
                    device
                        .device()
                        .flush_mapped_memory_ranges(&[mem_range]),
                    {
                        device.device().free_memory(dev_memory, device.allocator());
                        return Err(memory::MemoryError::Flush);
                    }
                );

                device.device().unmap_memory(dev_memory);
            }
        }

        Ok(Region {
            i_core: device.core().clone(),
            i_memory: dev_memory,
            i_size: size,
            i_flags: desc.flags()
        })
    }

    pub(crate) fn find_memory<'a, 'b : 'a>(hw: &'b hw::HWDevice, memory_bits: u32, properties: hw::MemoryProperty) -> Option<&'a hw::MemoryDescription> {
        let memory_filter = |m: &'b hw::MemoryDescription| -> Option<&'a hw::MemoryDescription> {
            if ((memory_bits >> m.index()) & 1) == 1
                && m.is_compatible(properties)
            {
                Some(m)
            } else {
                None
            }
        };

        hw.memory().find_map(memory_filter)
    }

    pub(crate) fn memory(&self) -> vk::DeviceMemory {
        self.i_memory
    }

    pub(crate) fn access<T, F>(&self, f: &mut F, offset: u64, size: u64) -> Result<(), memory::MemoryError>
    where
        F: FnMut(&mut [T]),
    {
        let data: *mut c_void = on_error_ret!(
            unsafe {
                self.i_core.device().map_memory(
                    self.i_memory,
                    offset,
                    size,
                    vk::MemoryMapFlags::empty(),
                )
            },
            memory::MemoryError::MapAccess
        );

        f(unsafe { std::slice::from_raw_parts_mut(data as *mut T, (size as usize)/std::mem::size_of::<T>()) });

        if !self
            .i_flags
            .contains(vk::MemoryPropertyFlags::HOST_COHERENT)
            && self
            .i_flags
            .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
        {
            let mem_range = vk::MappedMemoryRange {
                s_type: vk::StructureType::MAPPED_MEMORY_RANGE,
                p_next: ptr::null(),
                memory: self.i_memory,
                offset: offset,
                size: size,
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

        unsafe { self.i_core.device().unmap_memory(self.i_memory) };

        Ok(())
    }

    pub(crate) fn empty(core: &Arc<dev::Core>, size: u64) -> Region {
        Region {
            i_core: core.clone(),
            i_memory: vk::DeviceMemory::null(),
            i_size: size,
            i_flags: vk::MemoryPropertyFlags::empty()
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.i_memory == vk::DeviceMemory::null()
    }

    pub(crate) fn size(&self) -> u64 {
        self.i_size
    }
}

impl Drop for Region {
    fn drop(&mut self) {
        if !self.is_empty() {
            unsafe {
                self.i_core
                .device()
                .free_memory(self.i_memory, self.i_core.allocator());
            }
        }
    }
}

impl fmt::Debug for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Memory")
        .field("i_core", &self.i_core)
        .field("i_memory", &self.i_memory)
        .field("i_size", &self.i_size)
        .field("i_flags", &self.i_flags)
        .finish()
    }
}

#[inline]
fn offset(last: u64, alignment: u64) -> u64 {
    ((last % alignment != 0) as u64)*(alignment - last % alignment)
}