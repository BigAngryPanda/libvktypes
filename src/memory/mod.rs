//! Contains memory buffer, image etc.
//!
//! All types that are like "set of user data in memory" represented here

pub mod storage;
pub mod image;
pub mod framebuffer;
pub mod base;
pub mod vertex_buffer;

#[doc(hidden)]
pub use base::*;
#[doc(hidden)]
pub use storage::*;
#[doc(hidden)]
pub use image::*;
#[doc(hidden)]
pub use framebuffer::*;
#[doc(hidden)]
pub use vertex_buffer::*;

use crate::hw;

use std::error::Error;
use std::fmt;

/// Errors during [`Storage`](Storage) initialization and access
#[derive(Debug)]
pub enum MemoryError {
    /// Failed to [create](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateBuffer.html) buffer
    Buffer,
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
    Bind
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = match self {
            MemoryError::Buffer => {
                "Failed to create buffer (vkCreateBuffer call failed)"
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
            }
        };

        write!(f, "{:?}", err_msg)
    }
}

impl Error for MemoryError {}

/// Configuration struct for memory structs such as [`VertexBuffer`] of [`Storage`]
pub struct MemoryCfg<'a> {
    pub size: u64,
    pub properties: hw::MemoryProperty,
    pub shared_access: bool,
    pub transfer_src: bool,
    pub transfer_dst: bool,
    pub queue_families: &'a [u32]
}