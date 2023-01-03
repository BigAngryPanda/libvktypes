//! Vulkan Queue handler

use ash::vk;

use std::fmt;

use crate::dev;

/// Information about what queue to allocate
///
/// [`family_index`](crate::queue::QueueCfg::family_index)
/// **must be** one of the defined in [`DeviceCfg`](crate::dev::DeviceCfg)
///
/// [`queue_index`](crate::queue::QueueCfg::queue_index)
/// **must be** less than related queue count
#[doc = "See more: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetDeviceQueue.html>"]
#[derive(Debug)]
pub struct QueueCfg {
    pub family_index: u32,
    pub queue_index: u32,
}

pub struct Queue {
    i_queue: vk::Queue
}

impl Queue {
    pub fn new(dev: &dev::Device, cfg: &QueueCfg) -> Queue {
        Queue {
            i_queue: unsafe {
                dev.device().get_device_queue(cfg.family_index, cfg.queue_index)
            },
        }
    }
}

impl fmt::Debug for Queue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Queue")
        .field("i_queue", &(&self.i_queue as *const vk::Queue))
        .finish()
    }
}