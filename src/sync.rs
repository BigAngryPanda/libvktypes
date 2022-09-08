//! Syncronization primitives

use ash::vk;

use crate::dev;
use crate::on_error_ret;

use std::{error, fmt, ptr};

#[derive(Debug)]
pub enum SemaphoreError {
    Create,
}

impl fmt::Display for SemaphoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vkCreateSemaphore call failed")
    }
}

impl error::Error for SemaphoreError {}

pub struct SemaphoreType<'a> {
    pub device: &'a dev::Device,
}

pub struct Semaphore<'a> {
    i_device: &'a dev::Device,
    i_semaphore: vk::Semaphore,
}

impl<'a> Semaphore<'a> {
    pub fn new<'b>(cfg: &'b SemaphoreType<'a>) -> Result<Semaphore<'a>, SemaphoreError> {
        let dev = cfg.device.device();

        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
        };

        let semaphore = on_error_ret!(
            unsafe { dev.create_semaphore(&semaphore_create_info, None) },
            SemaphoreError::Create
        );

        Ok(Semaphore {
            i_device: cfg.device,
            i_semaphore: semaphore,
        })
    }

    #[doc(hidden)]
    pub fn semaphore(&self) -> vk::Semaphore {
        self.i_semaphore
    }
}

impl<'a> Drop for Semaphore<'a> {
    fn drop(&mut self) {
        unsafe {
            self.i_device
                .device()
                .destroy_semaphore(self.i_semaphore, None);
        }
    }
}

#[derive(Debug)]
pub enum FenceError {
    Create,
}

impl fmt::Display for FenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vkCreateFence call failed")
    }
}

impl error::Error for FenceError {}

pub struct FenceType<'a> {
    pub device: &'a dev::Device,
    pub signaled: bool,
}

pub struct Fence<'a> {
    i_device: &'a dev::Device,
    i_fence: vk::Fence,
}

impl<'a> Fence<'a> {
    pub fn new<'b>(cfg: &'b FenceType<'a>) -> Result<Fence<'a>, FenceError> {
        let dev = cfg.device.device();

        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: if cfg.signaled {
                vk::FenceCreateFlags::SIGNALED
            } else {
                vk::FenceCreateFlags::empty()
            },
        };

        let fence = on_error_ret!(
            unsafe { dev.create_fence(&fence_create_info, None) },
            FenceError::Create
        );

        Ok(Fence {
            i_device: cfg.device,
            i_fence: fence,
        })
    }

    #[doc(hidden)]
    pub fn fence(&self) -> vk::Fence {
        self.i_fence
    }
}

impl<'a> Drop for Fence<'a> {
    fn drop(&mut self) {
        unsafe {
            self.i_device
                .device()
                .destroy_fence(self.i_fence, None);
        }
    }
}