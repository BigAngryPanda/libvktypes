//! Vulkan Queue handler

use ash::vk;

use std::{fmt, ptr};
use std::sync::Arc;

use crate::{on_error_ret, data_ptr};
use crate::{dev, cmd, sync, swapchain};

pub struct ExecInfo<'a, 'b : 'a> {
    pub buffer: &'a cmd::ExecutableBuffer<'b>,
    pub wait_stage: cmd::PipelineStage,
    pub timeout: u64,
    pub wait: &'a [&'a sync::Semaphore<'b>],
    pub signal: &'a [&'a sync::Semaphore<'b>],
}

pub struct PresentInfo<'a, 'b : 'a> {
    pub swapchain: &'a swapchain::Swapchain,
    pub image_index: u32,
    pub wait: &'a [&'a sync::Semaphore<'b>]
}

#[derive(Debug)]
pub enum QueueError {
    /// Failed to
    /// [submit](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkQueueSubmit.html
    /// queue
    Execution,
    /// Failed to
    /// [create](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateFence.html)
    /// fence
    Fence,
    /// Execution time exceed max time
    Timeout,
    /// Failed to
    /// [present](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkQueuePresentKHR.html)
    /// image from swapchain
    Present
}

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
    i_core: Arc<dev::Core>,
    i_queue: vk::Queue
}

impl Queue {
    pub fn new(dev: &dev::Device, cfg: &QueueCfg) -> Queue {
        Queue {
            i_core: dev.core().clone(),
            i_queue: unsafe {
                dev.device().get_device_queue(cfg.family_index, cfg.queue_index)
            },
        }
    }

    /// Execute selected buffer
    pub fn exec(&self, info: &ExecInfo) -> Result<(), QueueError> {
        let dev = self.i_core.device();

        let fence_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags:  vk::FenceCreateFlags::empty()
        };

        let fence = on_error_ret!(
            unsafe { dev.create_fence(&fence_info, self.i_core.allocator()) },
            QueueError::Fence
        );

        let wait_sems: Vec<vk::Semaphore> = info.wait.iter().map(|s| s.semaphore()).collect();
        let sign_sems: Vec<vk::Semaphore> = info.signal.iter().map(|s| s.semaphore()).collect();

        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: wait_sems.len() as u32,
            p_wait_semaphores: data_ptr!(wait_sems),
            p_wait_dst_stage_mask: &info.wait_stage,
            command_buffer_count: 1,
            p_command_buffers: info.buffer.buffer(),
            signal_semaphore_count: sign_sems.len() as u32,
            p_signal_semaphores: data_ptr!(sign_sems),
        };

        unsafe {
            if dev.queue_submit(self.i_queue, &[submit_info], fence).is_err() {
               dev.destroy_fence(fence, self.i_core.allocator());
               return Err(QueueError::Execution);
            }
        }

        unsafe {
            if dev.wait_for_fences(&[fence], true, info.timeout).is_err() {
               dev.destroy_fence(fence, self.i_core.allocator());
               return Err(QueueError::Timeout);
            }
        }

        unsafe { dev.destroy_fence(fence, self.i_core.allocator()) };

        Ok(())
    }

    /// Present selected image from swapchain
    pub fn present(&self, info: &PresentInfo) -> Result<(), QueueError> {
        let semaphores: Vec<vk::Semaphore> = info.wait.iter().map(|s| s.semaphore()).collect();

        let present_info:vk::PresentInfoKHR = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: semaphores.len() as u32,
            p_wait_semaphores: data_ptr!(semaphores),
            swapchain_count: 1,
            p_swapchains: &info.swapchain.swapchain(),
            p_image_indices: &info.image_index,
            p_results: ptr::null_mut(),
        };

        on_error_ret!(unsafe { info.swapchain.loader().queue_present(self.i_queue, &present_info) }, QueueError::Present);

        Ok(())
    }
}

impl fmt::Debug for Queue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Queue")
        .field("i_queue", &(&self.i_queue as *const vk::Queue))
        .finish()
    }
}