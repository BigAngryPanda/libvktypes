//! Provides API to the selected GPU (Logical device)
//!
//! Instead of [hw module](crate::hw) `dev` represents logical level

use ash::vk;

use crate::{libvk, hw, alloc};
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
}

impl Core {
    fn new(device: ash::Device, callback: Option<alloc::Callback>) -> Core {
        Core {
            i_device: device,
            i_callback: callback,
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

/// Device configuration structure
///
/// Note: to prevent lifetime bounds [HWDevice](crate::hw::HWDevice) will be cloned
///
/// It is not optimal but maybe in the future it will be fixed
pub struct DeviceType<'a> {
    pub lib: &'a libvk::Instance,
    pub hw: &'a hw::HWDevice,
    pub queue_family_index: u32,
    pub priorities: &'a [f32],
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
    i_queue_index: u32,
    i_queue_count: u32,
    i_hw: hw::HWDevice,
    _marker: PhantomData<*const libvk::Instance>,
}

/// As Vulkan API specification demands instance must outlive device (and any other object which created via instance)
///
/// Hence lifetime requirements
impl Device {
    pub fn new(dev_type: &DeviceType) -> Result<Device, DeviceError> {
        let dev_queue_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index: dev_type.queue_family_index,
            queue_count: dev_type.priorities.len() as u32,
            p_queue_priorities: dev_type.priorities.as_ptr(),
        };

        let create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: 1,
            p_queue_create_infos: &dev_queue_info,
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
            i_queue_index: dev_type.queue_family_index,
            i_queue_count: dev_type.priorities.len() as u32,
            i_hw: dev_type.hw.clone(),
            _marker: PhantomData,
        })
    }

    /// Call this method to manually destroy library object
    pub fn destroy<T: Destroy>(&self, value: T) {
        value.destroy(&self.i_core);
    }

    #[doc(hidden)]
    pub fn core(&self) -> &Arc<Core> {
        &self.i_core
    }

    #[doc(hidden)]
    pub fn queue_index(&self) -> u32 {
        self.i_queue_index
    }

    #[doc(hidden)]
    pub fn queue_count(&self) -> u32 {
        self.i_queue_count
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

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { self.i_core.device().destroy_device(None) };
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
        unsafe { ManuallyDrop::drop(&mut self.i_value) };
        unsafe { ManuallyDrop::drop(&mut self.i_core) };
    }
}

impl<T: Destroy + fmt::Debug> Deref for DeviceCtx<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.i_value
    }
}