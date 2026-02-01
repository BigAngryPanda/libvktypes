//! Create layout
//!
//! Create bindings
//!
//! Create pipeline

pub mod layout;
pub mod bindings;
pub mod graphics;
pub mod compute;

#[doc(hidden)]
pub use layout::*;
#[doc(hidden)]
pub use bindings::*;

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
    /// Failed to create pipeline layout
    
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
