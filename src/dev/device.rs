//! Provides API to the selected GPU (Logical device)
//!
//! Instead of [hw module](crate::hw) `dev` represents logical level

use ash::vk;

use crate::{libvk, hw, alloc, queue, memory, sync, dev};
use crate::{on_error, on_error_ret};

use std::sync::Arc;
use std::ptr;

/// Device configuration structure
///
/// Note: on queue creation: every queue family in [`hw`](self::DeviceCfg::hw)
/// will be enabled and every queue within family will have equal priority
pub struct DeviceCfg<'a> {
    pub lib: &'a libvk::Instance,
    pub hw: &'a hw::HWDevice,
    pub extensions: &'a [*const i8],
    pub allocator: Option<alloc::Callback>,
}

#[derive(Debug)]
pub enum DeviceError {
    Creating,
}

/// Core structure of the library
///
/// `Device` represents logical device and provide API to the selected GPU
pub struct Device {
    i_core: Arc<dev::Core>,
    i_hw: hw::HWDevice,
}

impl Device {
    /// Create new [`Device`](self::Device) object according to [`DeviceCfg`](self::DeviceCfg)
    pub fn new(dev_type: &DeviceCfg) -> Result<Device, DeviceError> {
        let mut priorities: Vec<Vec<f32>> = Vec::new();

        let dev_queue_create_info: Vec<vk::DeviceQueueCreateInfo> = dev_type
            .hw
            .queues()
            .map(|info| {
                priorities.push(vec![1.0f32; info.count() as usize]);

                vk::DeviceQueueCreateInfo {
                    s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                    p_next: ptr::null(),
                    flags: vk::DeviceQueueCreateFlags::empty(),
                    queue_family_index: info.index(),
                    queue_count: info.count(),
                    p_queue_priorities: priorities.last().unwrap().as_ptr(),
                }
            })
            .collect();

        let create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: dev_queue_create_info.len() as u32,
            p_queue_create_infos: dev_queue_create_info.as_ptr(),
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(),
            enabled_extension_count: dev_type.extensions.len() as u32,
            pp_enabled_extension_names: dev_type.extensions.as_ptr(),
            p_enabled_features: dev_type.hw.features(),
        };

        let dev: ash::Device = on_error_ret!(
            unsafe { dev_type.lib.instance().create_device(dev_type.hw.device(), &create_info, None) },
            DeviceError::Creating
        );

        // Note: to prevent lifetime bounds [HWDevice](crate::hw::HWDevice) will be cloned
        //
        // It is not optimal but maybe in the future it will be fixed
        Ok(Device {
            i_core: Arc::new(dev::Core::new(dev, dev_type.allocator)),
            i_hw: dev_type.hw.clone()
        })
    }

    /// Create new queue
    ///
    /// For more information see [queue crate](crate::queue)
    pub fn get_queue(&self, cfg: &queue::QueueCfg) -> queue::Queue {
        queue::Queue::new(self, cfg)
    }

    /// Check if it is possible allocate memory with `cfg` from `desc`
    pub fn is_compatible(&self, desc: &hw::MemoryDescription, cfg: &memory::MemoryCfg) -> bool {
        let buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::BufferCreateFlags::empty(),
            size: cfg.size,
            usage: cfg.usage,
            sharing_mode: if cfg.shared_access { vk::SharingMode::CONCURRENT } else { vk::SharingMode::EXCLUSIVE },
            queue_family_index_count: cfg.queue_families.len() as u32,
            p_queue_family_indices: cfg.queue_families.as_ptr(),
        };

        let buffer: vk::Buffer = on_error!(
            unsafe { self.device().create_buffer(&buffer_info, None) },
            return false
        );

        let requirements: vk::MemoryRequirements = unsafe {
            self
                .device()
                .get_buffer_memory_requirements(buffer)
        };

        unsafe {
            self.i_core.device().destroy_buffer(buffer, self.i_core.allocator())
        };

        ((requirements.memory_type_bits >> desc.index()) & 1) == 1
            && desc.is_compatible(cfg.properties)
    }

    /// Return iterator over memories filtered by `f` and [compatibility](Device::is_compatible) with `cfg`
    pub fn filter_memory<'a, T>(&'a self, f: T, cfg: &'a memory::MemoryCfg) -> impl Iterator<Item = &'a hw::MemoryDescription>
    where
        T: Fn(&hw::MemoryDescription) -> bool
    {
        self.i_hw.filter_memory(move |m| f(m) && self.is_compatible(m, cfg))
    }

    /// Tries to find first suitable memory
    pub fn find_memory<'a, T>(&'a self, f: T, cfg: &'a memory::MemoryCfg) -> Option<&'a hw::MemoryDescription>
    where
        T: Fn(&hw::MemoryDescription) -> bool
    {
        self.filter_memory(f, cfg).next()
    }

    pub fn create_fence(&self, signaled: bool) -> Result<sync::Fence, sync::FenceError> {
        sync::Fence::new(&self, signaled)
    }

    pub fn create_semaphore(&self) -> Result<sync::Semaphore, sync::SemaphoreError> {
        sync::Semaphore::new(&self)
    }

    #[doc(hidden)]
    pub fn core(&self) -> &Arc<dev::Core> {
        &self.i_core
    }

    #[doc(hidden)]
    pub fn device(&self) -> &ash::Device {
        self.i_core.device()
    }

    #[doc(hidden)]
    pub fn allocator(&self) -> Option<&alloc::Callback> {
        self.i_core.allocator()
    }

    #[doc(hidden)]
    pub fn hw(&self) -> &hw::HWDevice {
        &self.i_hw
    }
}