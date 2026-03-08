use ash::vk;

use crate::dev::{
    Device,
    Core
};

use crate::{
    pipeline::LayoutError
};

use crate::{
    on_error_ret,
    data_ptr
};

use std::sync::Arc;

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

    /// `count` - hint for builder to preallocate memory
    /// for sets information
    pub fn with_sets(count: usize) -> PipelineLayoutBuilder {
        PipelineLayoutBuilder {
            push_constants: Vec::new(),
            bindings: vec![Vec::new(); count]
        }
    }

    pub fn push_constant(
        mut self,
        stage: ShaderStage,
        offset: u32,
        size: u32
    ) -> Self {
        self.push_constants.push(
            vk::PushConstantRange {
                stage_flags: stage,
                offset,
                size
            }
        );

        self
    }

    pub fn binding(
        mut self,
        set: usize,
        binding: u32,
        stage: ShaderStage,
        resource_type: DescriptorType,
        count: u32
    ) -> Self {
        if self.bindings.len() <= set {
            self.bindings.resize(set + 1, Vec::new());
        }

        self.bindings[set].push(BindingType { binding, resource_type, stage, count });

        self
    }

    pub fn build(&mut self, device: &Device) -> Result<PipelineLayout, LayoutError> {
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
            i_core: device.core().clone(),
            i_sets_layouts: sets_layout,
            i_layout: pipeline_layout,
            i_bindings: self.bindings.clone()
        })
    }
}

/*
    A pipeline layout describes all the resources that can be accessed by the pipeline
*/
#[derive(Debug)]
pub struct PipelineLayout {
    i_core: Arc<Core>,
    i_sets_layouts: Vec<vk::DescriptorSetLayout>,
    i_layout: vk::PipelineLayout,
    i_bindings: Vec<Vec<BindingType>>
}

impl PipelineLayout {
    pub(crate) fn bindings(&self) -> &Vec<Vec<BindingType>> {
        &self.i_bindings
    }

    pub(crate) fn layout(&self) -> vk::PipelineLayout {
        self.i_layout
    }

    pub(crate) fn sets_layouts(&self) -> &Vec<vk::DescriptorSetLayout> {
        &self.i_sets_layouts
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

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        let device = self.i_core.device();
        let alloc  = self.i_core.allocator();

        unsafe {
            for &set in &self.i_sets_layouts {
                device
                .destroy_descriptor_set_layout(set, alloc);
            }

            device.destroy_pipeline_layout(self.i_layout, alloc);
        }
    }
}