//! Allocator functions and types

/// Callback configuration
///
/// For now `Callback` must have static lifetime
#[doc = "See more: <https://docs.rs/ash/latest/ash/vk/struct.AllocationCallbacks.html>"]
pub type Callback = ash::vk::AllocationCallbacks<'static>;