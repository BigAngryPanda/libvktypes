//! Graphics pipeline and render pass
//!
//! # RenderPass
//! [`RenderPass`] represents context within graphics pipeline is executed
//!
//! It is defined by 3 components:
//! 1) [subpasses](SubpassInfo)
//! 2) [synchronization between subpasses](SubpassSync)
//! 3) [attachments](AttachmentInfo) which defines what for *all* images are used for

pub mod render_pass;
pub mod pipeline;

pub use crate::graphics::render_pass::*;
pub use crate::graphics::pipeline::*;