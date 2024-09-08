//! Syncronization primitives

use ash::vk;

use crate::dev;
use crate::on_error_ret;

use std::sync::Arc;
use std::{error, fmt, ptr};

use std::marker::PhantomData;

#[derive(Debug)]
pub enum SemaphoreError {
    Create,
}

impl fmt::Display for SemaphoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to create semaphore (vkCreateSemaphore call failed)")
    }
}

impl error::Error for SemaphoreError {}

#[derive(Debug)]
pub struct Semaphore {
    i_core: Arc<dev::Core>,
    i_semaphore: vk::Semaphore,
}

impl Semaphore {
    pub fn new(device: &dev::Device) -> Result<Semaphore, SemaphoreError> {
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
            _marker: PhantomData,
        };

        let semaphore = on_error_ret!(
            unsafe { device.device().create_semaphore(&semaphore_create_info, device.allocator()) },
            SemaphoreError::Create
        );

        Ok(Semaphore {
            i_core: device.core().clone(),
            i_semaphore: semaphore,
        })
    }

    #[doc(hidden)]
    pub fn semaphore(&self) -> vk::Semaphore {
        self.i_semaphore
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.i_core
                .device()
                .destroy_semaphore(self.i_semaphore, self.i_core.allocator());
        }
    }
}

#[derive(Debug)]
pub enum FenceError {
    Create,
}

impl fmt::Display for FenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to create fence (vkCreateFence call failed)")
    }
}

impl error::Error for FenceError {}

pub struct Fence {
    i_core: Arc<dev::Core>,
    i_fence: vk::Fence,
}

impl Fence {
    pub fn new(device: &dev::Device, signaled: bool) -> Result<Fence, FenceError> {
        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: if signaled {
                vk::FenceCreateFlags::SIGNALED
            } else {
                vk::FenceCreateFlags::empty()
            },
            _marker: PhantomData,
        };

        let fence = on_error_ret!(
            unsafe { device.device().create_fence(&fence_create_info, device.allocator()) },
            FenceError::Create
        );

        Ok(Fence {
            i_core: device.core().clone(),
            i_fence: fence,
        })
    }

    #[doc(hidden)]
    pub fn fence(&self) -> vk::Fence {
        self.i_fence
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.i_core
                .device()
                .destroy_fence(self.i_fence, self.i_core.allocator());
        }
    }
}