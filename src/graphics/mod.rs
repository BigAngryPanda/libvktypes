//! Graphics pipeline and render pass

use ash::vk;

pub mod render_pass;
pub mod pipeline;
pub mod resource;

#[doc(hidden)]
pub use crate::graphics::render_pass::*;
#[doc(hidden)]
pub use crate::graphics::pipeline::*;
#[doc(hidden)]
pub use resource::*;

/// ShaderStage specifies shader stage within single pipeline
///
#[doc = "Ash documentation about possible values <https://docs.rs/ash/latest/ash/vk/struct.ShaderStageFlags.html>"]
///
#[doc = "Vulkan documentation <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkShaderStageFlagBits.html>"]
pub type ShaderStage = vk::ShaderStageFlags;

/// `BindingCfg` gives pipeline information about how and what resource should be bind to the set in shader
///
/// Example for fragment shader
///
/// ```ignore
///     // ...
///
///     layout(set=X, binding=Y) uniform Data {
///         vec4 colour;
///     } data;
///
///     // ...
/// ```
///
/// As you may see `BindingCfg.0 == ResourceType::UNIFORM_BUFFER`
/// because we want to bind uniform buffer
///
/// Note: it also might be ResourceType::UNIFORM_BUFFER_DYNAMIC but they are not supported for now
///
/// `BindingCfg.1 == ShaderStage::FRAGMENT` (obviously, but it might be any valid combination with `FRAGMENT` flag)
///
/// `BindingCfg.2 == 1` as we have single struct (array of structs is not supported for now)
///
/// About `0` [count](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkDescriptorSetLayoutBinding.html)
///
/// Typically you should stay with [`Resource::layout`](crate::graphics::Resource::layout) method which will return you valid `BindingCfg`
///
/// Also pay attention that `BindingCfg` **does not** specify `X` and `Y` in set and binding
///
/// See more information about correct isage [here](crate::graphics::PipelineCfg)
pub type BindingCfg = (ResourceType, ShaderStage, u32);