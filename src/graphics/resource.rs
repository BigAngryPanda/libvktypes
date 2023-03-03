//! Contain trait that marks type can be used in shader (via binding point)
//!
//! Note: vertex buffer, depth buffer, image for drawing is sort of separate entity with different workflow
//! even if they are implemented trait (as they don't use binding points)

use ash::vk;

/// Marks that type can be used in shader via sets
///
/// Example
///
/// ```ignore
///     // ...
///
///     layout(set=X, binding=Y) uniform <glsl_type> <name>;
///
///     // ...
/// ```
///
/// More information can be found on [wiki](https://www.khronos.org/opengl/wiki/Layout_Qualifier_(GLSL))
///
/// Also see [`PipelineCfg`](crate::graphics::pipeline::PipelineCfg)
/// how to properly bind resources and determine `X` and `Y` in example
pub trait Resource {
    #[doc(hidden)]
    fn descriptor(&self) -> vk::DescriptorType;
}