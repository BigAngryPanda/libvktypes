//! Contains memory buffer, image etc.
//!
//! All types that are like "set of user data in memory" represented here

use ash::vk;

use crate::{hw, dev, swapchain};
use crate::on_error_ret;

use std::ptr;
use core::ffi::c_void;

// TODO mb rewrite it with separate flags?

/// Represents buffer usage flags
///
#[doc = "Possible values: <https://docs.rs/ash/latest/ash/vk/struct.BufferUsageFlags.html>"]
///
#[doc = "Vulkan documentation: <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkBufferUsageFlagBits.html>"]
pub type UsageFlags = vk::BufferUsageFlags;

/// Represents buffer access type
///
#[doc = "Possible values: <https://docs.rs/ash/latest/ash/vk/struct.SharingMode.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkSharingMode.html>"]
pub type SharingMode = vk::SharingMode;

/// Configuration of [`Memory`](Memory) struct
pub struct MemoryType<'a> {
    pub device: &'a dev::Device,
    pub size: u64,
    pub properties: hw::MemoryProperty,
    pub usage: UsageFlags,
    pub sharing_mode: SharingMode,
    pub queue_families: &'a [u32],
}

/// Errors during [`Memory`](Memory) initialization and access
#[derive(Debug)]
pub enum MemoryError {
    /// Failed to [create](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateBuffer.html) buffer
    Buffer,
    /// Failed to get suitable memory index
    ///
    /// In other words no memory is satisfying memory [`usage flags`](MemoryType::usage)
    NoMemoryType,
    /// Failed to [allocate](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkAllocateMemory.html) memory
    DeviceMemory,
    /// Failed to
    /// [map](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkMapMemory.html) memory
    MapAccess,
    /// Failed to
    /// [flush](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkFlushMappedMemoryRanges.html) memory
    Flush,
    /// Failed to
    /// [bind](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkBindBufferMemory.html) memory
    Bind,
}

/// Aligned region in memory with [specified](MemoryType) properties
pub struct Memory<'a> {
    i_device: &'a dev::Device,
    i_device_memory: vk::DeviceMemory,
    i_buffer: vk::Buffer,
    i_size: u64,
    i_flags: hw::MemoryProperty,
}

impl<'a> Memory<'a> {
    /// Allocate new region of memory
    ///
    /// Note: if memory is HOST_VISIBLE and is not HOST_COHERENT performs
    /// [map_memory](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkMapMemory.html)
    /// and
    /// [flush](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkFlushMappedMemoryRanges.html)
    /// which may result in [errors](MemoryError::MapAccess)
    pub fn allocate(mem_cfg: &'a MemoryType) -> Result<Memory<'a>, MemoryError> {
        let buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::BufferCreateFlags::empty(),
            size: mem_cfg.size,
            usage: mem_cfg.usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: mem_cfg.queue_families.len() as u32,
            p_queue_family_indices: mem_cfg.queue_families.as_ptr(),
        };

        let buffer: vk::Buffer = on_error_ret!(
            unsafe { mem_cfg.device.device().create_buffer(&buffer_info, None) },
            MemoryError::Buffer
        );

        let requirements: vk::MemoryRequirements = unsafe {
            mem_cfg.device.device().get_buffer_memory_requirements(buffer)
        };

        let memory_filter = |m : &hw::MemoryDescription| -> Option<u32> {
            if ((requirements.memory_type_bits >> m.index()) & 1) == 1 && m.is_compatible(mem_cfg.properties) {
                Some(m.index())
            }
            else {
                None
            }
        };

        let mem_index: u32 = match mem_cfg.device.hw().memory().find_map(memory_filter) {
            Some(val) => val,
            None => return Err(MemoryError::NoMemoryType),
        };

        let memory_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: requirements.size,
            memory_type_index: mem_index,
        };

        let dev_memory:vk::DeviceMemory = on_error_ret!(
            unsafe { mem_cfg.device.device().allocate_memory(&memory_info, None) },
            MemoryError::DeviceMemory
        );

        // Without coherency we have to manually synchronize memory between host and device
        if !mem_cfg.properties.contains(vk::MemoryPropertyFlags::HOST_COHERENT)
            && mem_cfg.properties.contains(vk::MemoryPropertyFlags::HOST_VISIBLE) {
            let mem_range = vk::MappedMemoryRange {
                s_type: vk::StructureType::MAPPED_MEMORY_RANGE,
                p_next: ptr::null(),
                memory: dev_memory,
                offset: 0,
                size: vk::WHOLE_SIZE
            };

            unsafe {
                on_error_ret!(
                    mem_cfg.device.device().map_memory(dev_memory, 0, mem_cfg.size, vk::MemoryMapFlags::empty()),
                    MemoryError::MapAccess
                );

                on_error_ret!(
                    mem_cfg.device.device().flush_mapped_memory_ranges(&[mem_range]),
                    MemoryError::Flush
                );

                mem_cfg.device.device().unmap_memory(dev_memory);
            }
        }

        on_error_ret!(
            unsafe { mem_cfg.device.device().bind_buffer_memory(buffer, dev_memory, 0) },
            MemoryError::Bind
        );

        Ok(
			Memory {
				i_device: mem_cfg.device,
				i_device_memory: dev_memory,
				i_buffer: buffer,
				i_size: mem_cfg.size,
				i_flags: mem_cfg.properties,
			}
		)
    }

    /// Performs action on mutable memory
    ///
    /// If memory is not coherent performs
    /// [vkFlushMappedMemoryRanges](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkFlushMappedMemoryRanges.html)
    ///
    /// In other words makes host memory changes available to device
	pub fn write<F>(&self, f: &mut F) -> Result<(), MemoryError>
    where F: FnMut(&mut [u8])
    {
        let data: *mut c_void = on_error_ret!(
            unsafe {
                self.i_device.device().map_memory(self.i_device_memory, 0, self.i_size, vk::MemoryMapFlags::empty())
            },
            MemoryError::MapAccess
        );

        f(unsafe {std::slice::from_raw_parts_mut(data as *mut u8, self.i_size as usize)});

        if !self.i_flags.contains(vk::MemoryPropertyFlags::HOST_COHERENT) {
            let mem_range = vk::MappedMemoryRange {
                s_type: vk::StructureType::MAPPED_MEMORY_RANGE,
                p_next: ptr::null(),
                memory: self.i_device_memory,
                offset: 0,
                size: vk::WHOLE_SIZE
            };

            on_error_ret!(
                unsafe {
                    self.i_device.device().flush_mapped_memory_ranges(&[mem_range])
                },
                MemoryError::Flush
            );
        }

        unsafe { self.i_device.device().unmap_memory(self.i_device_memory) };

        Ok(())
    }

    /// Return copy of buffer's memory
    ///
    /// If memory is not coherent performs
    /// [vkInvalidateMappedMemoryRanges](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkInvalidateMappedMemoryRanges.html)
    ///
    /// I.e. makes device memory changes available to host (compare with [Memory::write()] method)
    ///
    /// Note: on failure return same error [MemoryError::Flush]
    pub fn read(&self) -> Result<&[u8], MemoryError>
    {
        if !self.i_flags.contains(vk::MemoryPropertyFlags::HOST_COHERENT) {
            let mem_range = vk::MappedMemoryRange {
                s_type: vk::StructureType::MAPPED_MEMORY_RANGE,
                p_next: ptr::null(),
                memory: self.i_device_memory,
                offset: 0,
                size: vk::WHOLE_SIZE
            };

            on_error_ret!(
                unsafe {
                    self.i_device.device().invalidate_mapped_memory_ranges(&[mem_range])
                },
                MemoryError::Flush
            );
        }

        let data: *mut c_void = on_error_ret!(
            unsafe {
                self.i_device.device().map_memory(self.i_device_memory, 0, self.i_size, vk::MemoryMapFlags::empty())
            },
            MemoryError::MapAccess
        );

        let result: &[u8] = unsafe {std::slice::from_raw_parts_mut(data as *mut u8, self.i_size as usize)};

        unsafe { self.i_device.device().unmap_memory(self.i_device_memory) };

        Ok(result)
    }

    /// Return size of the buffer in bytes
    pub fn size(&self) -> u64 {
        self.i_size
    }

    #[doc(hidden)]
    pub fn buffer(&self) -> vk::Buffer {
        self.i_buffer
    }
}

impl<'a> Drop for Memory<'a> {
    fn drop(&mut self) {
        unsafe {
            self.i_device.device().destroy_buffer(self.i_buffer, None);
            self.i_device.device().free_memory(self.i_device_memory, None);
        };
    }
}

/// Errors during [`Image`] initialization and access
#[derive(Debug)]
pub enum ImageError {
    GetImages,
    ImageView,
}

/// Images represent multidimensional - up to 3 - arrays of data
///
/// Instead of [`Memory`] `Image` are more specified
pub struct Image<'a> {
    i_dev: &'a dev::Device,
    i_image_view: vk::ImageView,
}

impl<'a> Image<'a> {
    #[doc(hidden)]
    fn new(device: &'a dev::Device, img: vk::Image, img_format: vk::Format) -> Result<Image<'a>, ImageError> {
        let image_info:vk::ImageViewCreateInfo = vk::ImageViewCreateInfo {
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
            unsafe { device.device().create_image_view(&image_info, None) },
            ImageError::ImageView
        );

        Ok(
            Image {
                i_dev: device,
                i_image_view: img_view
            }
        )
    }
}

impl<'a> Drop for Image<'a> {
    fn drop(&mut self) {
        unsafe { self.i_dev.device().destroy_image_view(self.i_image_view, None) };
    }
}

pub struct ImageListType<'a> {
    pub device: &'a dev::Device,
    pub swapchain: &'a swapchain::Swapchain
}

/// Collection of [`Images`](Image)
pub struct ImageList<'a>(Vec::<Image<'a>>);

impl<'a> ImageList<'a> {
    /// Retrieves [image handlers](Image) from [`Swapchain`](crate::swapchain::Swapchain)
    pub fn from_swapchain(swp_type: &'a ImageListType) -> Result<ImageList<'a>, ImageError> {
        let swapchain_images = on_error_ret!(
            unsafe {
                swp_type.swapchain.loader().get_swapchain_images(swp_type.swapchain.swapchain())
            },
            ImageError::GetImages
        );

        let mut img_view = Vec::<Image<'a>>::new();

        for img in swapchain_images {
            match Image::new(swp_type.device, img, swp_type.swapchain.format()) {
                Ok(val) => img_view.push(val),
                Err(e) => return Err(e),
            }
        }

        Ok(ImageList(img_view))
    }

    /// Number of images in list
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Is list empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return iterator over images in list
    pub fn images(&self) -> impl Iterator<Item = &Image> {
        self.0.iter()
    }
}