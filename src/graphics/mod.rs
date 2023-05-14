//! Graphics pipeline and render pass

use ash::vk;

pub mod render_pass;
pub mod pipeline;
pub mod resource;
pub mod vertex_view;

#[doc(hidden)]
pub use crate::graphics::render_pass::*;
#[doc(hidden)]
pub use crate::graphics::pipeline::*;
#[doc(hidden)]
pub use resource::*;
#[doc(hidden)]
pub use vertex_view::*;

/// ShaderStage specifies shader stage within single pipeline
///
#[doc = "Ash documentation about possible values <https://docs.rs/ash/latest/ash/vk/struct.ShaderStageFlags.html>"]
///
#[doc = "Vulkan documentation <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkShaderStageFlagBits.html>"]
pub type ShaderStage = vk::ShaderStageFlags;