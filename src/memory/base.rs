//! Generic interface for various buffer classes
//! (such as [vertex buffer](crate::memory::vertex_buffer::VertexBuffer), [storage](crate::memory::storage::Storage) etc.)
use ash::vk;

use crate::{on_error, on_error_ret};
use crate::{dev, hw, memory};

use core::ffi::c_void;
use std::sync::Arc;
use std::ptr;

#[doc(hidden)]
pub fn is_compatible(
    device: &dev::Device,
    desc: &hw::MemoryDescription,
    cfg: &memory::MemoryCfg,
    usage: vk::BufferUsageFlags
) -> bool {
    let buffer_info = vk::BufferCreateInfo {
        s_type: vk::StructureType::BUFFER_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::BufferCreateFlags::empty(),
        size: cfg.size,
        usage: usage,
        sharing_mode: if cfg.shared_access { vk::SharingMode::CONCURRENT } else { vk::SharingMode::EXCLUSIVE },
        queue_family_index_count: cfg.queue_families.len() as u32,
        p_queue_family_indices: cfg.queue_families.as_ptr(),
    };

    let buffer: vk::Buffer = on_error!(
        unsafe { device.device().create_buffer(&buffer_info, device.allocator()) },
        return false
    );

    let requirements: vk::MemoryRequirements = unsafe {
        device
            .device()
            .get_buffer_memory_requirements(buffer)
    };

    unsafe {
        device.device().destroy_buffer(buffer, device.allocator())
    };

    ((requirements.memory_type_bits >> desc.index()) & 1) == 1
        && desc.is_compatible(cfg.properties)
}

#[doc(hidden)]
pub fn filter_memory<'a, T>(
    device: &'a dev::Device,
    f: T,
    cfg: &'a memory::MemoryCfg,
    usage: vk::BufferUsageFlags
) -> impl Iterator<Item = &'a hw::MemoryDescription>
where
    T: Fn(&hw::MemoryDescription) -> bool
{
    device.hw().filter_memory(move |m| f(m) && is_compatible(device, m, cfg, usage))
}

#[doc(hidden)]
pub fn find_memory<'a, T>(
    device: &'a dev::Device,
    f: T,
    cfg: &'a memory::MemoryCfg,
    usage: vk::BufferUsageFlags
) -> Option<&'a hw::MemoryDescription>
where
    T: Fn(&hw::MemoryDescription) -> bool
{
    filter_memory(device, f, cfg, usage).next()
}

#[doc(hidden)]
pub struct BaseStorage {
    i_core: Arc<dev::Core>,
    i_device_memory: vk::DeviceMemory,
    i_buffer: vk::Buffer,
    i_size: u64,
    i_flags: hw::MemoryProperty
}

#[doc(hidden)]
impl BaseStorage {
    pub fn new(
        device: &dev::Device,
        memory: &hw::MemoryDescription,
        mem_cfg: &memory::MemoryCfg,
        usage: vk::BufferUsageFlags
    ) -> Result<BaseStorage, memory::MemoryError> {
        let buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::BufferCreateFlags::empty(),
            size: mem_cfg.size,
            usage: usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: mem_cfg.queue_families.len() as u32,
            p_queue_family_indices: mem_cfg.queue_families.as_ptr(),
        };

        let buffer: vk::Buffer = on_error_ret!(
            unsafe { device.device().create_buffer(&buffer_info, device.allocator()) },
            memory::MemoryError::Buffer
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
                    return Err(memory::MemoryError::DeviceMemory);
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
                        return Err(memory::MemoryError::MapAccess);
                    }
                );

                on_error!(
                    device
                        .device()
                        .flush_mapped_memory_ranges(&[mem_range]),
                    {
                        device.device().free_memory(dev_memory, device.allocator());
                        device.device().destroy_buffer(buffer, device.allocator());
                        return Err(memory::MemoryError::Flush);
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
            memory::MemoryError::Bind
        );

        Ok(BaseStorage {
            i_core: device.core().clone(),
            i_device_memory: dev_memory,
            i_buffer: buffer,
            i_size: mem_cfg.size,
            i_flags: mem_cfg.properties
        })
    }

    pub fn write<T, F>(&self, f: &mut F) -> Result<(), memory::MemoryError>
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
            memory::MemoryError::MapAccess
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
                memory::MemoryError::Flush
            );
        }

        unsafe { self.i_core.device().unmap_memory(self.i_device_memory) };

        Ok(())
    }

    pub fn read(&self) -> Result<&[u8], memory::MemoryError> {
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
                memory::MemoryError::Flush
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
            memory::MemoryError::MapAccess
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

impl Drop for BaseStorage {
    fn drop(&mut self) {
        unsafe {
            self.i_core.device().destroy_buffer(self.i_buffer, self.i_core.allocator());
            self.i_core
                .device()
                .free_memory(self.i_device_memory, self.i_core.allocator());
        };
    }
}