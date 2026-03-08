//! Create layout
//!
//! Create bindings
//!
//! Create pipeline

use ash::vk;

pub mod layout;
pub mod bindings;
pub mod graphics;
pub mod compute;

#[doc(hidden)]
pub use layout::*;
#[doc(hidden)]
pub use bindings::*;
#[doc(hidden)]
pub use compute::*;
#[doc(hidden)]
pub use graphics::*;

/// Describe how vertices should be assembled into primitives
///
#[doc = "Possible values: <https://docs.rs/ash/latest/ash/vk/struct.PrimitiveTopology.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkPrimitiveTopology.html>"]
pub type Topology = vk::PrimitiveTopology;

/// Specifies which triangles will be discarderd based on their orientation
///
#[doc = "Possible values: <https://docs.rs/ash/latest/ash/vk/struct.CullModeFlags.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkCullModeFlagBits.html>"]
pub type CullMode = vk::CullModeFlags;

/// ShaderStage specifies shader stage within single pipeline
///
#[doc = "Ash documentation about possible values <https://docs.rs/ash/latest/ash/vk/struct.ShaderStageFlags.html>"]
///
#[doc = "Vulkan documentation <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkShaderStageFlagBits.html>"]
pub type ShaderStage = vk::ShaderStageFlags;

#[derive(Debug)]
pub enum LayoutError {
    DescriptorSet,
    Layout
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutError::DescriptorSet => write!(f, "vkCreateDescriptorSetLayout call failed"),
            LayoutError::Layout => write!(f, "vkCreatePipelineLayout call failed"),
        }
    }
}

#[derive(Debug)]
pub enum BindingError {
    DescriptorPool,
    DescriptorAllocation
}

impl std::fmt::Display for BindingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BindingError::DescriptorPool => write!(f, "vkCreateDescriptorPool call failed"),
            BindingError::DescriptorAllocation => write!(f, "vkDescriptorSetAllocateInfo call failed")
        }
    }
}

#[derive(Debug)]
pub enum PipelineError {
    PipelineCache,
    /// Failed to create pipeline
    Pipeline,
    ComputePipeline
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineError::PipelineCache => write!(f, "vkCreatePipelineCache call failed"),
            PipelineError::Pipeline => write!(f, "vkCreateGraphicsPipelines call failed"),
            PipelineError::ComputePipeline => write!(f, "vkCreateGraphicsPipelines call failed")
        }
    }
}

impl std::error::Error for PipelineError { }
