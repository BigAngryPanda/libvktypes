use ash::vk;

use crate::{
    dev,
    on_error_ret,
    shader,
    pipeline
};

use std::sync::Arc;
use core::ffi::c_char;

pub struct ComputePipelineBuilder {
    comp_shader: vk::ShaderModule,
    comp_entry: *const c_char,
}

impl ComputePipelineBuilder {
    pub fn new() -> ComputePipelineBuilder {
        ComputePipelineBuilder {
            comp_shader: vk::ShaderModule::null(),
            comp_entry: std::ptr::null()
        }
    }

    /// Must be called
    ///
    /// `shader` must outlive builder
    pub fn compute_shader(&mut self, shader: &shader::Shader) -> &mut Self {
        self.comp_shader = shader.module();
        self.comp_entry = shader.entry().as_ptr();

        self
    }

    /// Try to create pipeline
    pub fn build(&self,
        device: &dev::Device,
        layout: &pipeline::PipelineLayout
    ) -> Result<ComputePipeline, pipeline::PipelineError> {
        use std::marker::PhantomData;

        let pipeline_shader = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::COMPUTE,
            module: self.comp_shader,
            p_name: self.comp_entry,
            p_specialization_info: std::ptr::null(),
            _marker: PhantomData,
        };

        let pipeline_info = vk::ComputePipelineCreateInfo {
            s_type: vk::StructureType::COMPUTE_PIPELINE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineCreateFlags::empty(),
            stage: pipeline_shader,
            layout: layout.layout(),
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: 0,
            _marker: std::marker::PhantomData,
        };

        let pipelines = unsafe { on_error_ret!(
            device.device().create_compute_pipelines(vk::PipelineCache::null(), &[pipeline_info], device.allocator()),
            pipeline::PipelineError::ComputePipeline
        )};

        Ok(ComputePipeline {
            i_core: device.core().clone(),
            i_pipeline: pipelines[0],
        })
    }
}

pub struct ComputePipeline {
    i_core: Arc<dev::Core>,
    i_pipeline: vk::Pipeline
}

impl ComputePipeline {
    pub(crate) fn pipeline(&self) -> vk::Pipeline {
        self.i_pipeline
    }
}

impl Drop for ComputePipeline {
    fn drop(&mut self) {
        let device = self.i_core.device();
        let alloc  = self.i_core.allocator();

        unsafe {
            device.destroy_pipeline(self.i_pipeline, alloc);
        }
    }
}
