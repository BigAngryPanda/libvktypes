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

/// Requested queue configuration
///
/// Example
/// ```rust
/// use libvktypes::dev::QueueFamilyCfg;
///
/// let cfg = QueueFamilyCfg {
///     queue_family_index: 0,
///     priorities: &[1.0, 0.5],
/// };
/// ```
///
/// Device will use queue family with index `0`
/// and `2` queues from queue family with priorities `1.0` and `0.5` respectively
#[derive(Debug)]
pub struct QueueFamilyCfg<'a> {
    /// Which queue family [`Device`] should use
    ///
    /// See [`QueueFamilyDescription::index`](crate::hw::QueueFamilyDescription::index)
    pub queue_family_index: u32,
    /// `priorities.len()` defines how many queues will be used by [`Device`]
    ///
    /// `priorities` data defines their relative priorities within [`Device`]
    ///
    /// `priorities.len()` **must be** less or equal to the
    /// [number of queues](crate::hw::QueueFamilyDescription::count)
    ///
    /// Also about priorities see vulkan
    /// [documentation](https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#devsandqueues-priority)
    pub priorities: &'a [f32],
}

/// Device configuration structure
///
/// Note: to prevent lifetime bounds [HWDevice](crate::hw::HWDevice) will be cloned
///
/// It is not optimal but maybe in the future it will be fixed
pub struct DeviceCfg<'a> {
    pub lib: &'a libvk::Instance,
    pub hw: &'a hw::HWDevice,
    pub queues_cfg: &'a [QueueFamilyCfg<'a>],
    pub extensions: &'a [*const i8],
    pub allocator: Option<alloc::Callback>,
}

#[derive(Debug)]
pub enum DeviceError {
    Creating,
}

/// Information about what queue family [`Device`] uses
#[derive(Debug)]
pub struct QueueInfo {
    i_index: u32,
    i_count: u32,
}

impl QueueInfo {
    /// Queue family index
    pub fn index(&self) -> u32 {
        self.i_index
    }

    /// How many queues in use
    pub fn count(&self) -> u32 {
        self.i_count
    }
}

/// Core structure of the library
///
/// `Device` represents logical device and provide API to the selected GPU
pub struct Device {
    i_core: Arc<Core>,
    i_queues: Vec<QueueInfo>,
    i_hw: hw::HWDevice,
    _marker: PhantomData<*const libvk::Instance>,
}

/// As Vulkan API specification demands instance must outlive device (and any other object which created via instance)
///
/// Hence lifetime requirements
impl Device {
    pub fn new(dev_type: &DeviceCfg) -> Result<Device, DeviceError> {
        let dev_queue_info: Vec<QueueInfo> = dev_type
            .queues_cfg
            .iter()
            .map(|info| QueueInfo {
                i_index: info.queue_family_index,
                i_count: info.priorities.len() as u32,
            })
            .collect();

        let dev_queue_create_info: Vec<vk::DeviceQueueCreateInfo> = dev_type
            .queues_cfg
            .iter()
            .map(|info| vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: info.queue_family_index,
                queue_count: info.priorities.len() as u32,
                p_queue_priorities: info.priorities.as_ptr(),
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
            i_queues: dev_queue_info,
            i_hw: dev_type.hw.clone(),
            _marker: PhantomData,
        })
    }

    /// Call this method to manually destroy library object
    pub fn destroy<T: Destroy>(&self, value: T) {
        value.destroy(&self.i_core);
    }

    /// Return information about i-th queue family in use
    ///
    /// `i` **must be** less than [`Device::queue_family_count`] length
    pub fn queue(&self, i: u32) -> &QueueInfo {
        &self.i_queues[i as usize]
    }

    /// Return information about how many queue families in use
    pub fn queue_family_count(&self) -> u32 {
        self.i_queues.len() as u32
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