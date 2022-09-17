//! Contains memory buffer, image etc.
//!
//! All types that are like "set of user data in memory" represented here

use ash::vk;

use crate::on_error_ret;
use crate::{dev, graphics, hw, surface, swapchain};

use core::ffi::c_void;
use std::error::Error;
use std::fmt;
use std::ptr;
use std::ops::Index;

// TODO mb rewrite it with separate flags?

/// Represents buffer usage flags
///
#[doc = "Possible values: <https://docs.rs/ash/latest/ash/vk/struct.BufferUsageFlags.html>"]
///
#[doc = "Vulkan documentation: <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkBufferUsageFlagBits.html>"]
pub type BufferUsageFlags = vk::BufferUsageFlags;

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
    pub usage: BufferUsageFlags,
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
            mem_cfg
                .device
                .device()
                .get_buffer_memory_requirements(buffer)
        };

        let memory_filter = |m: &hw::MemoryDescription| -> Option<u32> {
            if ((requirements.memory_type_bits >> m.index()) & 1) == 1
                && m.is_compatible(mem_cfg.properties)
            {
                Some(m.index())
            } else {
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

        let dev_memory: vk::DeviceMemory = on_error_ret!(
            unsafe { mem_cfg.device.device().allocate_memory(&memory_info, None) },
            MemoryError::DeviceMemory
        );

        // Without coherency we have to manually synchronize memory between host and device
        if !mem_cfg
            .properties
            .contains(vk::MemoryPropertyFlags::HOST_COHERENT)
            && mem_cfg
                .properties
                .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
        {
            let mem_range = vk::MappedMemoryRange {
                s_type: vk::StructureType::MAPPED_MEMORY_RANGE,
                p_next: ptr::null(),
                memory: dev_memory,
                offset: 0,
                size: vk::WHOLE_SIZE,
            };

            unsafe {
                on_error_ret!(
                    mem_cfg.device.device().map_memory(
                        dev_memory,
                        0,
                        mem_cfg.size,
                        vk::MemoryMapFlags::empty()
                    ),
                    MemoryError::MapAccess
                );

                on_error_ret!(
                    mem_cfg
                        .device
                        .device()
                        .flush_mapped_memory_ranges(&[mem_range]),
                    MemoryError::Flush
                );

                mem_cfg.device.device().unmap_memory(dev_memory);
            }
        }

        on_error_ret!(
            unsafe {
                mem_cfg
                    .device
                    .device()
                    .bind_buffer_memory(buffer, dev_memory, 0)
            },
            MemoryError::Bind
        );

        Ok(Memory {
            i_device: mem_cfg.device,
            i_device_memory: dev_memory,
            i_buffer: buffer,
            i_size: mem_cfg.size,
            i_flags: mem_cfg.properties,
        })
    }

    /// Performs action on mutable memory
    ///
    /// If memory is not coherent performs
    /// [vkFlushMappedMemoryRanges](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkFlushMappedMemoryRanges.html)
    ///
    /// In other words makes host memory changes available to device
    pub fn write<T, F>(&self, f: &mut F) -> Result<(), MemoryError>
    where
        F: FnMut(&mut [T]),
    {
        let data: *mut c_void = on_error_ret!(
            unsafe {
                self.i_device.device().map_memory(
                    self.i_device_memory,
                    0,
                    self.i_size,
                    vk::MemoryMapFlags::empty(),
                )
            },
            MemoryError::MapAccess
        );

        f(unsafe { std::slice::from_raw_parts_mut(data as *mut T, (self.i_size as usize)/std::mem::size_of::<T>()) });

        if !self
            .i_flags
            .contains(vk::MemoryPropertyFlags::HOST_COHERENT)
        {
            let mem_range = vk::MappedMemoryRange {
                s_type: vk::StructureType::MAPPED_MEMORY_RANGE,
                p_next: ptr::null(),
                memory: self.i_device_memory,
                offset: 0,
                size: vk::WHOLE_SIZE,
            };

            on_error_ret!(
                unsafe {
                    self.i_device
                        .device()
                        .flush_mapped_memory_ranges(&[mem_range])
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
    pub fn read(&self) -> Result<&[u8], MemoryError> {
        if !self
            .i_flags
            .contains(vk::MemoryPropertyFlags::HOST_COHERENT)
        {
            let mem_range = vk::MappedMemoryRange {
                s_type: vk::StructureType::MAPPED_MEMORY_RANGE,
                p_next: ptr::null(),
                memory: self.i_device_memory,
                offset: 0,
                size: vk::WHOLE_SIZE,
            };

            on_error_ret!(
                unsafe {
                    self.i_device
                        .device()
                        .invalidate_mapped_memory_ranges(&[mem_range])
                },
                MemoryError::Flush
            );
        }

        let data: *mut c_void = on_error_ret!(
            unsafe {
                self.i_device.device().map_memory(
                    self.i_device_memory,
                    0,
                    self.i_size,
                    vk::MemoryMapFlags::empty(),
                )
            },
            MemoryError::MapAccess
        );

        let result: &[u8] =
            unsafe { std::slice::from_raw_parts_mut(data as *mut u8, self.i_size as usize) };

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
            self.i_device
                .device()
                .free_memory(self.i_device_memory, None);
        };
    }
}

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

/// Errors during [`Image`] initialization and access
#[derive(Debug)]
pub enum ImageError {
    GetImages,
    Creating,
    ImageView,
    NoMemoryType,
    DeviceMemory,
    Bind,
}

pub struct ImageType<'a> {
    pub device: &'a dev::Device,
    pub queue_families: &'a [u32],
    pub format: surface::ImageFormat,
    pub extent: surface::Extent3D,
    pub usage: ImageUsageFlags,
    pub layout: graphics::ImageLayout,
    pub aspect: ImageAspect,
    pub properties: hw::MemoryProperty,
}

/// Images represent multidimensional - up to 3 - arrays of data
///
/// Instead of [`Memory`] `Image` are more specified
pub struct Image<'a> {
    i_dev: &'a dev::Device,
    i_image: vk::Image,
    i_image_view: vk::ImageView,
    i_image_memory: vk::DeviceMemory,
}

impl<'a> Image<'a> {
    pub fn new(cfg: &ImageType<'a>) -> Result<Image<'a>, ImageError> {
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
            unsafe { cfg.device.device().create_image(&image_info, None) },
            ImageError::Creating
        );

        let requirements: vk::MemoryRequirements = unsafe {
            cfg
                .device
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

        let mem_index: u32 = match cfg.device.hw().memory().find_map(memory_filter) {
            Some(val) => val,
            None => return Err(ImageError::NoMemoryType),
        };

        let memory_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: requirements.size,
            memory_type_index: mem_index,
        };

        let img_memory: vk::DeviceMemory = on_error_ret!(
            unsafe { cfg.device.device().allocate_memory(&memory_info, None) },
            ImageError::DeviceMemory
        );

        on_error_ret!(
            unsafe {
                cfg
                    .device
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
            unsafe { cfg.device.device().create_image_view(&iv_info, None) },
            ImageError::ImageView
        );

        Ok(
            Image {
                i_dev: cfg.device,
                i_image: img,
                i_image_view: img_view,
                i_image_memory: img_memory,
            }
        )
    }

    #[doc(hidden)]
    fn from_raw(
        device: &'a dev::Device,
        img: vk::Image,
        img_format: vk::Format,
    ) -> Result<Image<'a>, ImageError> {
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
            unsafe { device.device().create_image_view(&image_info, None) },
            ImageError::ImageView
        );

        Ok(Image {
            i_dev: device,
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

impl<'a> Drop for Image<'a> {
    fn drop(&mut self) {
        unsafe {
            self.i_dev
                .device()
                .destroy_image_view(self.i_image_view, None);

            self.i_dev
                .device()
                .destroy_image(self.i_image, None);

            if self.i_image_memory != vk::DeviceMemory::null() {
                self.i_dev
                    .device()
                    .free_memory(self.i_image_memory, None);
            }
        };
    }
}

pub struct ImageListType<'a> {
    pub device: &'a dev::Device,
    pub swapchain: &'a swapchain::Swapchain,
}

/// Collection of [`Images`](Image)
pub struct ImageList<'a>(Vec<Image<'a>>);

impl<'a> ImageList<'a> {
    /// Retrieves [image handlers](Image) from [`Swapchain`](crate::swapchain::Swapchain)
    pub fn from_swapchain<'b>(
        swp_type: &'b ImageListType<'a>,
    ) -> Result<ImageList<'a>, ImageError> {
        let swapchain_images = on_error_ret!(
            unsafe {
                swp_type
                    .swapchain
                    .loader()
                    .get_swapchain_images(swp_type.swapchain.swapchain())
            },
            ImageError::GetImages
        );

        let mut img_view = Vec::<Image<'a>>::new();

        for img in swapchain_images {
            match Image::from_raw(swp_type.device, img, swp_type.swapchain.format()) {
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

impl<'a> Index<usize> for ImageList<'a> {
    type Output = Image<'a>;

    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

#[derive(Debug)]
pub enum FramebufferError {
    Framebuffer,
}

impl fmt::Display for FramebufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vkCreateFramebuffer call failed")
    }
}

impl Error for FramebufferError {}

/// Framebuffer represents a collection of specific memory attachments that a render pass instance uses
pub struct Framebuffer<'a> {
    i_dev: &'a dev::Device,
    i_frame: vk::Framebuffer,
    i_extent: vk::Extent2D,
}

impl<'a> Framebuffer<'a> {
    #[doc(hidden)]
    fn new(
        dev: &'a dev::Device,
        img: vk::ImageView,
        extent: vk::Extent2D,
        rp: vk::RenderPass,
    ) -> Result<Framebuffer<'a>, FramebufferError> {
        let create_info = vk::FramebufferCreateInfo {
            s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FramebufferCreateFlags::empty(),
            render_pass: rp,
            attachment_count: 1,
            p_attachments: &img,
            width: extent.width,
            height: extent.height,
            layers: 1,
        };

        let framebuffer = on_error_ret!(
            unsafe { dev.device().create_framebuffer(&create_info, None) },
            FramebufferError::Framebuffer
        );

        Ok(Framebuffer {
            i_dev: dev,
            i_frame: framebuffer,
            i_extent: extent,
        })
    }

    #[doc(hidden)]
    pub fn framebuffer(&self) -> vk::Framebuffer {
        self.i_frame
    }

    #[doc(hidden)]
    pub fn extent(&self) -> vk::Extent2D {
        self.i_extent
    }
}

impl<'a> Drop for Framebuffer<'a> {
    fn drop(&mut self) {
        unsafe {
            self.i_dev.device().destroy_framebuffer(self.i_frame, None);
        }
    }
}

pub struct FramebufferType<'a> {
    pub device: &'a dev::Device,
    pub render_pass: &'a graphics::RenderPass<'a>,
    pub images: &'a ImageList<'a>,
    pub extent: surface::Extent2D,
}

/// Collection of [`Framebuffers`](Framebuffer)
pub struct FramebufferList<'a>(Vec<Framebuffer<'a>>);

impl<'a> FramebufferList<'a> {
    pub fn new<'b>(cfg: &'b FramebufferType<'a>) -> Result<FramebufferList<'a>, FramebufferError> {
        let mut list: Vec<Framebuffer<'a>> = Vec::new();

        for img in cfg.images.images() {
            list.push(on_error_ret!(
                Framebuffer::new(
                    cfg.device,
                    img.view(),
                    cfg.extent,
                    cfg.render_pass.render_pass()
                ),
                FramebufferError::Framebuffer
            ));
        }

        Ok(FramebufferList(list))
    }

    /// Return iterator over framebuffers
    pub fn framebuffers(&self) -> impl Iterator<Item = &Framebuffer> {
        self.0.iter()
    }
}

impl<'a> Index<usize> for FramebufferList<'a> {
    type Output = Framebuffer<'a>;

    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}