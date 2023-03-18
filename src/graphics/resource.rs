//! Contain trait that marks type can be used in shader (via binding point)
//!
//! Note: vertex buffers, images are used for drawing but sort of separate entities with different workflow
//! so they don't implement the trait (despite they are used by shaders one way or another)

use ash::vk;

use crate::graphics;

pub type ResourceType = vk::DescriptorType;

/// Marks that type can be used in shader via sets
///
/// Example
///
/// ```ignore
///     // ...
///
///     layout(set=X, binding=Y) <type (e.g. uniform)> <struct_name> {
///         <filed_type> <field_name>;
///     } <var_name>[<count, omit if count == 1>];
///
///     // ...
///
/// ```
///
/// Another example
///
/// ```ignore
///     // ...
///
///     layout(set=0, binding=0) uniform Data {
///         vec4 colour;
///     } data[2];
///
///     // ...
/// ```
///
/// More information can be found on [wiki](https://www.khronos.org/opengl/wiki/Layout_Qualifier_(GLSL))
///
/// Also see [`PipelineCfg`](crate::graphics::pipeline::PipelineCfg)
/// how to properly bind resources and determine `X` and `Y` in example
pub trait Resource {
    fn resource_type(&self) -> ResourceType;
    /// How many array elements within single `resource`
    fn count(&self) -> u32;
    /// How this resource should be bind to the selected set in shader
    fn layout(&self, stage: graphics::ShaderStage) -> graphics::BindingCfg;
    #[doc(hidden)]
    fn buffer(&self) -> vk::Buffer;
    #[doc(hidden)]
    fn size(&self) -> u64;
}