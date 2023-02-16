//! More specific version of [`Memory`](crate::memory::memory::Memory)

use ash::vk;

use crate::on_error_ret;
use crate::{dev, graphics, hw, surface};

use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::ptr;

/// Represents image usage flags
///
#[doc = "Possible values: <https://docs.rs/ash/latest/ash/vk/struct.ImageUsageFlags.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkImageUsageFlagBits.html>"]
pub type ImageUsageFlags = vk::ImageUsageFlags;

/// Represents which aspects of an image will be used
///
#[doc = "Possible values: <https://docs.rs/ash/latest/ash/vk/struct.ImageAspectFlags.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkImageAspectFlagBits.html>"]
pub type ImageAspect = vk::ImageAspectFlags;

/// Image formats
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.Format.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkFormat.html>"]
pub type ImageFormat = vk::Format;

/// Errors during [`Image`] initialization and access
#[derive(Debug)]
pub enum ImageError {
    Creating,
    ImageView,
    DeviceMemory,
    Bind
}

impl fmt::Display for ImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = match self {
            ImageError::Creating => {
                "Failed to create image (vkCreateImage call failed)"
            },
            ImageError::ImageView => {
                "Failed to create image view (vkCreateImageView call failed)"
            },
            ImageError::DeviceMemory => {
                "Failed to allocate memory for image"
            },
            ImageError::Bind => {
                "Failed to bind image memory"
            }
        };

        write!(f, "{:?}", err_msg)
    }
}

impl Error for ImageError {}

pub struct ImageCfg<'a> {
    pub queue_families: &'a [u32],
    pub format: ImageFormat,
    pub extent: surface::Extent3D,
    pub usage: ImageUsageFlags,
    pub layout: graphics::ImageLayout,
    pub aspect: ImageAspect,
    pub properties: hw::MemoryProperty,
}

/// Images represent multidimensional - up to 3 - arrays of data
pub struct Image {
    i_core: Arc<dev::Core>,
    i_image: vk::Image,
    i_image_view: vk::ImageView,
    i_image_memory: vk::DeviceMemory,
}

impl Image {
    pub fn new(device: &dev::Device, cfg: &ImageCfg) -> Result<Image, ImageError> {
        let image_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            format: cfg.format,
            extent: cfg.extent,
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: cfg.usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: cfg.queue_families.len() as u32,
            p_queue_family_indices: cfg.queue_families.as_ptr(),
            initial_layout: cfg.layout,
        };

        let img = on_error_ret!(
            unsafe { device.device().create_image(&image_info, device.allocator()) },
            ImageError::Creating
        );

        let requirements: vk::MemoryRequirements = unsafe {
            device
                .device()
                .get_image_memory_requirements(img)
        };

        let memory_filter = |m: &hw::MemoryDescription| -> Option<u32> {
            if ((requirements.memory_type_bits >> m.index()) & 1) == 1
                && m.is_compatible(cfg.properties)
            {
                Some(m.index())
            } else {
                None
            }
        };

        let mem_index: u32 = match device.hw().memory().find_map(memory_filter) {
            Some(val) => val,
            None => return Err(ImageError::Bind),
        };

        let memory_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: requirements.size,
            memory_type_index: mem_index,
        };

        let img_memory = on_error_ret!(
            unsafe { device.device().allocate_memory(&memory_info, device.allocator()) },
            ImageError::DeviceMemory
        );

        on_error_ret!(
            unsafe {
                device
                    .device()
                    .bind_image_memory(img, img_memory, 0)
            },
            ImageError::Bind
        );

        let iv_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageViewCreateFlags::empty(),
            view_type: vk::ImageViewType::TYPE_2D,
            format: cfg.format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: cfg.aspect,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            image: img,
        };

        let img_view = on_error_ret!(
            unsafe { device.device().create_image_view(&iv_info, device.allocator()) },
            ImageError::ImageView
        );

        Ok(
            Image {
                i_core: device.core().clone(),
                i_image: img,
                i_image_view: img_view,
                i_image_memory: img_memory,
            }
        )
    }

    #[doc(hidden)]
    pub fn preallocated(
        core: &Arc<dev::Core>,
        img: vk::Image,
        img_format: vk::Format,
    ) -> Result<Image, ImageError> {
        let image_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageViewCreateFlags::empty(),
            view_type: vk::ImageViewType::TYPE_2D,
            format: img_format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            image: img,
        };

        let img_view = on_error_ret!(
            unsafe { core.device().create_image_view(&image_info, core.allocator()) },
            ImageError::ImageView
        );

        Ok(Image {
            i_core: core.clone(),
            i_image: img,
            i_image_view: img_view,
            i_image_memory: vk::DeviceMemory::null(),
        })
    }

    #[doc(hidden)]
    pub fn view(&self) -> vk::ImageView {
        self.i_image_view
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            self.i_core
                .device()
                .destroy_image_view(self.i_image_view, self.i_core.allocator());

            if self.i_image_memory != vk::DeviceMemory::null() {
                self.i_core
                    .device()
                    .destroy_image(self.i_image, self.i_core.allocator());

                self.i_core
                    .device()
                    .free_memory(self.i_image_memory, self.i_core.allocator());
            }
        };
    }
}