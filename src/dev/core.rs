use crate::{libvk, alloc};

use std::marker::PhantomData;
use std::fmt;

pub(crate) struct Core {
    i_device: ash::Device,
    i_callback: Option<alloc::Callback>,
    _marker: PhantomData<*const libvk::Instance>
}

impl Core {
    pub(crate) fn new(device: ash::Device, callback: Option<alloc::Callback>) -> Core {
        Core {
            i_device: device,
            i_callback: callback,
            _marker: PhantomData
        }
    }

    pub(crate) fn device(&self) -> &ash::Device {
        &self.i_device
    }

    pub(crate) fn allocator(&self) -> Option<&alloc::Callback> {
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