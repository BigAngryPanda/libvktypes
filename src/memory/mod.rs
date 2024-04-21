//! Contains various buffer types
//!
//! All types that are like "region of user data in memory" are represented here
//!
//! Notable exeption of this is [`framebuffer`](crate::memory::Framebuffer)

pub mod memory;
pub mod image;
pub mod framebuffer;
pub mod view;
pub(crate) mod region;

#[doc(hidden)]
pub use memory::*;
#[doc(hidden)]
pub use image::*;
#[doc(hidden)]
pub use framebuffer::*;
#[doc(hidden)]
pub use view::*;
pub(crate) use region::*;

use std::error::Error;
use std::fmt;

/// Layout of image and image subresources
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.ImageLayout.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkImageLayout.html>"]
pub type ImageLayout = ash::vk::ImageLayout;

/// Errors during memory allocation, initialization and access
#[derive(Debug)]
pub enum MemoryError {
    /// Failed to [create](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateBuffer.html) buffer
    Buffer,
    /// Failed to find suitable memory
    NoSuitableMemory,
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
    /// Failed to
    /// [invalidate mapped memory range](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkInvalidateMappedMemoryRanges.html)
    Sync,
    /// Failed to
    /// [allocate](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateImage.html) image
    Image,
    /// Failed to
    /// [create](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateImageView.html) image view
    ImageView,
    /// Failed to
    /// [bind](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkBindImageMemory.html) image memory
    ImageBind
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = match self {
            MemoryError::Buffer => {
                "Failed to create buffer (vkCreateBuffer call failed)"
            },
            MemoryError::NoSuitableMemory => {
                "Failed to find suitable memory"
            },
            MemoryError::DeviceMemory => {
                "Failed to allocate memory for buffer (vkAllocateMemory call failed)"
            },
            MemoryError::MapAccess => {
                "Failed to map memory (vkMapMemory call failed)"
            },
            MemoryError::Flush => {
                "Failed to flush memory (vkFlushMappedMemoryRanges call failed)"
            },
            MemoryError::Bind => {
                "Failed to bind memory (vkBindBufferMemory call failed)"
            },
            MemoryError::Sync => {
                "Failed to invalidate mapped memory range (vkInvalidateMappedMemoryRanges call failed)"
            },
            MemoryError::Image => {
                "Failed to create image (vkCreateImage call failed)"
            },
            MemoryError::ImageView => {
                "Failed to create image view (vkCreateImageView call failed)"
            },
            MemoryError::ImageBind => {
                "Failed to bind image memory (vkBindImageMemory call failed)"
            }
        };

        write!(f, "{:?}", err_msg)
    }
}

impl Error for MemoryError {}