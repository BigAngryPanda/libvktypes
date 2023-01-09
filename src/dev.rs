//! Provides API to the selected GPU (Logical device)
//!
//! Instead of [hw module](crate::hw) `dev` represents logical level

use ash::vk;

use crate::{libvk, hw, alloc, queue};
use crate::on_error_ret;

use std::marker::PhantomData;
use std::ptr;
use std::sync::Arc;
use std::ops::Deref;
use std::fmt;
use std::mem::ManuallyDrop;

#[doc(hidden)]
pub struct Core {
    i_device: ash::Device,
    i_callback: Option<alloc::Callback>,
    _marker: PhantomData<*const libvk::Instance>
}

impl Core {
    fn new(device: ash::Device, callback: Option<alloc::Callback>) -> Core {
        Core {
            i_device: device,
            i_callback: callback,
            _marker: PhantomData
        }
    }

    pub fn device(&self) -> &ash::Device {
        &self.i_device
    }

    pub fn callback(&self) -> Option<&alloc::Callback> {
        self.i_callback.as_ref()
    }
}

impl fmt::Debug for Core {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Core")
        .field("i_device", &(&self.i_device as *const ash::Device))
        .field("i_callback", &self.i_callback)
        .finish()
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        unsafe { self.i_device.destroy_device(self.i_callback.as_ref()) };
    }
}

/// Device configuration structure
///
/// Note: to prevent lifetime bounds [HWDevice](crate::hw::HWDevice) will be cloned
///
/// It is not optimal but maybe in the future it will be fixed
///
/// Note on queue creation: every queue family in [`hw`](self::DeviceCfg::hw)
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
    i_core: Arc<Core>,
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

        Ok(Device {
            i_core: Arc::new(Core::new(dev, dev_type.allocator)),
            i_hw: dev_type.hw.clone()
        })
    }

    /// Call this method to manually destroy library object
    pub fn destroy<T: Destroy>(&self, value: T) {
        value.destroy(&self.i_core);
    }

    /// Create new queue
    ///
    /// For more information see [queue crate](crate::queue)
    pub fn get_queue(&self, cfg: &queue::QueueCfg) -> queue::Queue {
        queue::Queue::new(self, cfg)
    }

    /// Manually destroy library object
    pub fn manually_destroy<T: Destroy>(&self, obj: T) {
        obj.destroy(&self.i_core);
    }

    #[doc(hidden)]
    pub fn core(&self) -> &Arc<Core> {
        &self.i_core
    }

    #[doc(hidden)]
    pub fn device(&self) -> &ash::Device {
        self.i_core.device()
    }

    #[doc(hidden)]
    pub fn allocator(&self) -> Option<&alloc::Callback> {
        self.i_core.callback()
    }

    #[doc(hidden)]
    pub fn hw(&self) -> &hw::HWDevice {
        &self.i_hw
    }
}

/// Marks that objects can be destroyed by [`Device`]
pub trait Destroy {
    #[doc(hidden)]
    fn destroy(&self, core: &Core);
}

/// Provides smart ponter-like behaviour by destroying Vulkan object in [`Drop`]
#[derive(Debug)]
pub struct DeviceCtx<T: Destroy + fmt::Debug> {
    i_core: ManuallyDrop<Arc<Core>>,
    i_value: ManuallyDrop<T>,
}

impl <T: Destroy + fmt::Debug> DeviceCtx<T> {
    /// Consume `value` and return [`DeviceCtx`]
    pub fn new(device: &Device, value: T) -> DeviceCtx<T> {
        DeviceCtx {
            i_core: ManuallyDrop::new(device.core().clone()),
            i_value: ManuallyDrop::new(value),
        }
    }

    /// Consume [`DeviceCtx`] and return value
    ///
    /// Note: after calling this method it becomes your responsibility to destroy value
    ///
    /// Destructor will *not* be called
    pub fn leak(mut self) -> T {
        unsafe { ManuallyDrop::drop(&mut self.i_core) };

        let val: T = unsafe { ManuallyDrop::take(&mut self.i_value) };

        std::mem::forget(self);

        val
    }
}

impl<T: Destroy + fmt::Debug> Drop for DeviceCtx<T> {
    fn drop(&mut self) {
        self.i_value.destroy(self.i_core.as_ref());
        unsafe {
            ManuallyDrop::drop(&mut self.i_value);
            ManuallyDrop::drop(&mut self.i_core);
        };
    }
}

impl<T: Destroy + fmt::Debug> Deref for DeviceCtx<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.i_value
    }
}