use ash::vk;

use crate::{
    dev::Device,
    pipeline::LayoutError
};

use crate::{
    on_error_ret,
    data_ptr
};

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

/// Specifies how pipeline should treat region of memory
///
#[doc = "Ash documentation about possible values <https://docs.rs/ash/latest/ash/vk/struct.DescriptorType.html>"]
///
#[doc = "Vulkan documentation <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkDescriptorType.html>"]
pub type DescriptorType = vk::DescriptorType;

#[derive(Debug, Clone, Copy)]
pub(crate) struct BindingType {
    pub binding: u32,
    pub resource_type: DescriptorType,
    pub stage: ShaderStage,
    pub count: u32,
}

#[derive(Debug)]
pub struct PipelineLayoutBuilder {
    push_constants: Vec<vk::PushConstantRange>,
    bindings: Vec<Vec<BindingType>>
}

impl PipelineLayoutBuilder {
    pub fn new() -> PipelineLayoutBuilder {
        PipelineLayoutBuilder {
            push_constants: Vec::new(),
            bindings: Vec::new()
        }
    }

    pub fn push_constant(
        &mut self,
        stage: ShaderStage,
        offset: u32,
        size: u32
    ) -> &mut Self {
        self.push_constants.push(
            vk::PushConstantRange {
                stage_flags: stage,
                offset,
                size
            }
        );

        self
    }

    /// Allocate memory for sets bindings
    ///
    /// You must do it before calling [`binding`]
    ///
    /// Calling again will invalidate current bindings
    pub fn sets(&mut self, count: usize) -> &mut Self {
        self.bindings = vec![Vec::new(); count];

        self
    }

    pub fn binding(
        &mut self,
        set: usize,
        binding: u32,
        stage: ShaderStage,
        resource_type: DescriptorType,
        count: u32
    ) -> &mut Self {
        self.bindings[set].push(BindingType { binding, resource_type, stage, count });

        self
    }

    pub fn build(self, device: &Device) -> Result<PipelineLayout, LayoutError> {
        let mut sets_layout: Vec<vk::DescriptorSetLayout> = Vec::new();

        for bindings in &self.bindings {
            match create_set_layout(device, bindings) {
                Ok(set) => sets_layout.push(set),
                Err(_) => {
                    clear_sets_layout(device, &sets_layout);
                    return Err(LayoutError::DescriptorSet);
                }
            }
        };

        let layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: sets_layout.len() as u32,
            p_set_layouts: data_ptr!(sets_layout),
            push_constant_range_count: self.push_constants.len() as u32,
            p_push_constant_ranges: data_ptr!(self.push_constants),
            _marker: std::marker::PhantomData,
        };

        let pipeline_layout = unsafe { on_error_ret!(
            device.device().create_pipeline_layout(&layout_create_info, device.allocator()),
            LayoutError::Layout
        )};

        Ok(PipelineLayout {
            sets_layouts: sets_layout,
            layout: pipeline_layout,
            bindings: self.bindings
        })
    }
}

/*
    A pipeline layout describes all the resources that can be accessed by the pipeline
*/
#[derive(Debug)]
pub struct PipelineLayout {
    pub(crate) sets_layouts: Vec<vk::DescriptorSetLayout>,
    pub(crate) layout: vk::PipelineLayout,
    pub(crate) bindings: Vec<Vec<BindingType>>
}

impl PipelineLayout {
    pub(crate) fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }

    pub(crate) fn sets_layouts(&self) -> &Vec<vk::DescriptorSetLayout> {
        &self.sets_layouts
    }
}

fn create_set_layout(
    device: &Device,
    resources: &[BindingType]
) -> ash::prelude::VkResult<vk::DescriptorSetLayout> {
    use std::marker::PhantomData;

    let bindings: Vec<vk::DescriptorSetLayoutBinding> = resources.iter().map(
        |binding_cfg| vk::DescriptorSetLayoutBinding {
            binding: binding_cfg.binding,
            descriptor_type: binding_cfg.resource_type,
            descriptor_count: binding_cfg.count,
            stage_flags: binding_cfg.stage,
            p_immutable_samplers: std::ptr::null(),
            _marker: PhantomData,
        }
    ).collect();

    let desc_layout_info = vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: bindings.len() as u32,
        p_bindings: bindings.as_ptr(),
        _marker: PhantomData,
    };

    unsafe {
        device.device().create_descriptor_set_layout(&desc_layout_info, device.allocator())
    }
}

fn clear_sets_layout(
    device: &Device,
    sets: &Vec<vk::DescriptorSetLayout>)
{
    unsafe {
        for &set in sets {
            device
            .device()
            .destroy_descriptor_set_layout(set, device.allocator());
        }
    }
}
