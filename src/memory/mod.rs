//! Contains memory buffer, image etc.
//!
//! All types that are like "set of user data in memory" represented here

pub mod memory;
pub mod image;
pub mod framebuffer;

pub use memory::*;
pub use image::*;
pub use framebuffer::*;