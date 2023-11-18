//! Contain "view-like" type can be used in shader (via binding point)
//!
//! Note: vertex buffers, images are used for drawing but sort of separate entities with different workflow
//! (despite they are used by shaders one way or another)

use ash::vk;

use crate::{graphics, memory};

pub type ResourceType = vk::DescriptorType;

/// `Resource` contains information about how to use buffer in shader layouts
///
/// Example in glsl
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
/// More information can be found on [wiki](https://www.khronos.org/opengl/wiki/Layout_Qualifier_(GLSL))
///
/// Also see [`PipelineCfg`](crate::graphics::pipeline::PipelineCfg)
/// how to properly bind resources and determine `X` and `Y` in example
///
/// # Packing
///
/// As in example below resource may contain more than one buffer
///
/// ```ignore
///     // ...
///
///     layout(set=0, binding=0) uniform Data {
///         vec4 colour;
///     } data[N];
///
///     // ...
/// ```
///
/// It is your responsibility to guarantee that `[Resource::count()] == N`
///
/// According to
/// spec
/// [1](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkDescriptorSetLayoutBinding.html)
/// [2](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkWriteDescriptorSet.html#_description)
/// if `[Resource::count()] == 0` then you must not use corresponding binding
///
/// See [more](crate::graphics::PipelineCfg)
///
/// Also resource stage must match with actuall pipeline stage
///
/// For example, accessing fragment stage without `ShaderStage::FRAGMENT` will lead to the error
#[derive(Debug, Clone)]
pub struct Resource<'a> {
    i_buffers: Vec<memory::View<'a>>,
    i_desc_type: vk::DescriptorType,
    i_stage: vk::ShaderStageFlags
}

impl<'a> Resource<'a> {
    /// Create new resource from selected views
    pub fn from_memory(
        buffers: &[memory::View<'a>],
        resource_type: ResourceType,
        stage: graphics::ShaderStage
    ) -> Resource<'a> {
        Resource {
            i_buffers: Vec::from(buffers),
            i_desc_type: resource_type,
            i_stage: stage
        }
    }

    /// Create new resource with no views
    ///
    /// Main purpose for such struct is to skip selected binding
    ///
    /// However you have to specify type and stage
    ///
    /// Also you must not use such resource for actuall binding
    ///
    /// See also [`PipelineCfg`](crate::graphics::PipelineCfg)
    ///
    /// You may also create it from `new()` just by passing empty array
    pub fn empty(
        resource_type: ResourceType,
        stage: graphics::ShaderStage
    ) -> Resource<'a> {
        Resource::from_memory(&[], resource_type, stage)
    }

    /// How resource should be used in pipeline
    pub fn resource_type(&self) -> ResourceType {
        self.i_desc_type
    }

    /// How many buffers elements within single `resource`
    pub fn count(&self) -> u32 {
        self.i_buffers.len() as u32
    }

    /// Return underlying views
    pub fn views(&self) -> &Vec<memory::View> {
        &self.i_buffers
    }

    /// What stages are available for the resource
    pub fn stage(&self) -> vk::ShaderStageFlags {
        self.i_stage
    }

    /// Return selected view
    pub fn view(&self, index: usize) -> memory::View {
        self.i_buffers[index]
    }
}