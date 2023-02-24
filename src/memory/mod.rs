//! Contains memory buffer, image etc.
//!
//! All types that are like "set of user data in memory" represented here

pub mod storage;
pub mod image;
pub mod framebuffer;

pub use storage::*;
pub use image::*;
pub use framebuffer::*;