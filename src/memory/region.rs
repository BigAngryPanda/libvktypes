use ash::vk;

use core::ffi::c_void;
use std::sync::Arc;
use std::fmt;
use std::marker::PhantomData;

use crate::{
    on_error,
    on_error_ret
};
use crate::{
    dev,
    hw,
    memory
};

use std::ptr;

pub(crate) struct Region {
    i_core: Arc<dev::Core>,
    i_memory: vk::DeviceMemory,
    i_size: u64,
    i_flags: hw::MemoryProperty
}

impl Region {
    pub(crate) fn allocate(device: &dev::Device, size: u64, desc: &hw::MemoryDescription) -> Result<Region, memory::MemoryError> {
        let memory_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: size,
            memory_type_index: desc.index(),
            _marker: PhantomData,
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
                _marker: PhantomData,
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

    pub(crate) fn memory(&self) -> vk::DeviceMemory {
        self.i_memory
    }

    pub(crate) fn access<T, F>(&self, f: &mut F, offset: u64, size: u64, allocated_size: u64) -> Result<(), memory::MemoryError>
    where
        F: FnMut(&mut [T]),
    {
        let data = self.map_memory(offset, size, allocated_size)?;

        f(data);

        let result = if !self
            .i_flags
            .contains(vk::MemoryPropertyFlags::HOST_COHERENT)
            && self
            .i_flags
            .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
        {
            self.flush(offset, size)
        }
        else {
            Ok(())
        };

        self.unmap_memory();

        result
    }

    pub(crate) fn map_memory<T>(&self, offset: u64, size: u64, allocated_size: u64) -> Result<&mut [T], memory::MemoryError> {
        let data: *mut c_void = on_error_ret!(
            unsafe {
                self.i_core.device().map_memory(
                    self.i_memory,
                    offset,
                    allocated_size,
                    vk::MemoryMapFlags::empty(),
                )
            },
            memory::MemoryError::MapAccess
        );

        Ok(unsafe { std::slice::from_raw_parts_mut(data as *mut T, (size as usize)/std::mem::size_of::<T>()) })
    }

    pub(crate) fn flush(&self, offset: u64, size: u64) -> Result<(), memory::MemoryError> {
        let mem_range = vk::MappedMemoryRange {
            s_type: vk::StructureType::MAPPED_MEMORY_RANGE,
            p_next: ptr::null(),
            memory: self.i_memory,
            offset,
            size,
            _marker: PhantomData,
        };

        on_error_ret!(
            unsafe {
                self.i_core
                .device()
                .flush_mapped_memory_ranges(&[mem_range])
            },
            memory::MemoryError::Flush
        );

        Ok(())
    }

    pub(crate) fn sync(&self, offset: u64, size: u64) -> Result<(), memory::MemoryError> {
        let mem_range = vk::MappedMemoryRange {
            s_type: vk::StructureType::MAPPED_MEMORY_RANGE,
            p_next: ptr::null(),
            memory: self.i_memory,
            offset,
            size,
            _marker: PhantomData,
        };

        on_error_ret!(
            unsafe {
                self.i_core
                .device()
                .invalidate_mapped_memory_ranges(&[mem_range])
            },
            memory::MemoryError::Sync
        );

        Ok(())
    }

    pub(crate) fn unmap_memory(&self) {
        unsafe { self.i_core.device().unmap_memory(self.i_memory) };
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
