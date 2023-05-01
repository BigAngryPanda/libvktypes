//! Contains various buffer types
//!
//! All types that are like "region of user data in memory" are represented here
//!
//! Notable exeption of this is [`framebuffer`](crate::memory::Framebuffer)

pub mod memory;
pub mod image;
pub mod framebuffer;
pub mod view;

#[doc(hidden)]
pub use memory::*;
#[doc(hidden)]
pub use image::*;
#[doc(hidden)]
pub use framebuffer::*;
#[doc(hidden)]
pub use view::*;