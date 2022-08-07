//! Represent pipeline and its configuration
//!
//! Note: only [memory](crate::resorces::memory::Memory) with memory::UsageFlags::STORAGE_BUFFER is allowed

use ash::vk;

use crate::dev;
use crate::memory;
use crate::shader;

use crate::on_error_ret;

use std::ptr;

pub struct PipelineType<'a> {
    pub device: &'a dev::Device<'a>,
    pub buffers: &'a [&'a memory::Memory<'a>],
    pub shader: &'a shader::Shader<'a>,
    pub push_constant_size : u32,
}

#[derive(Debug)]
pub enum PipelineError {
    DescriptorPool,
    DescriptorSetLayout,
    DescriptorSet,
    PipelineLayout,
    PipelineCache,
    Pipeline
}

/// Represents compute pipeline
pub struct Pipeline<'a> {
    i_dev:             &'a dev::Device<'a>,
    i_pipeline_layout: vk::PipelineLayout,
    i_desc_set_layout: vk::DescriptorSetLayout,
    i_desc_set:        vk::DescriptorSet,
    i_desc_pool:       vk::DescriptorPool,
    i_pipeline:        vk::Pipeline,
    i_pipeline_cache:  vk::PipelineCache,
}

// TODO provide dynamic buffer binding
// TODO shader module must outlive pipeline?
impl<'a> Pipeline<'a> {
    pub fn new(pipe_type: &'a PipelineType) -> Result<Pipeline<'a>, PipelineError> {
        let desc_size:[vk::DescriptorPoolSize; 1] =
        [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: pipe_type.buffers.len() as u32,
            }
        ];

        let pool_size: u32 = 1;

        // So max_sets is how many *sets* we can possibly allocate
        // While PoolSize defines how many *descriptors* we can allocate
        // within single set ?
        let desc_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorPoolCreateFlags::empty(),
            max_sets: pool_size,
            pool_size_count: desc_size.len() as u32,
            p_pool_sizes: desc_size.as_ptr(),
        };

        let desc_pool = on_error_ret!(
            unsafe { pipe_type.device.device().create_descriptor_pool(&desc_info, None) },
            PipelineError::DescriptorPool
        );

        let bindings: Vec<vk::DescriptorSetLayoutBinding> = pipe_type.buffers.iter().enumerate().map(
            |(i, _)| vk::DescriptorSetLayoutBinding {
                binding: i as u32,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                p_immutable_samplers: ptr::null()
            }
        ).collect();

        let desc_layout_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            binding_count: bindings.len() as u32,
            p_bindings: bindings.as_ptr(),
        };

        let desc_set_layout = on_error_ret!(
            unsafe { pipe_type.device.device().create_descriptor_set_layout(&desc_layout_info, None) },
            PipelineError::DescriptorSetLayout
        );

        let push_const_range = vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::COMPUTE,
            offset: 0,
            size: pipe_type.push_constant_size,
        };

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: 1,
            p_set_layouts: &desc_set_layout,
            push_constant_range_count: if pipe_type.push_constant_size != 0 { 1 } else { 0 },
            p_push_constant_ranges: if pipe_type.push_constant_size != 0 { &push_const_range } else { ptr::null() }
        };

        let pipeline_layout = on_error_ret!(
            unsafe { pipe_type.device.device().create_pipeline_layout(&pipeline_layout_info, None) },
            PipelineError::PipelineLayout
        );

        let alloc_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_pool: desc_pool,
            descriptor_set_count: 1,
            p_set_layouts: &desc_set_layout
        };

        let desc_set = on_error_ret!(
            unsafe { pipe_type.device.device().allocate_descriptor_sets(&alloc_info) },
            PipelineError::DescriptorSet
        );

        let mut offset_counter = 0u64;
        let mut buffer_descs: Vec<vk::DescriptorBufferInfo> = Vec::new();

        for buffer in pipe_type.buffers {
            buffer_descs.push(
                    vk::DescriptorBufferInfo {
                    buffer: buffer.buffer(),
                    offset: offset_counter,
                    range: vk::WHOLE_SIZE
                }
            );

            offset_counter += buffer.size();
        }

        // TODO big question can we update set with single vk::WriteDescriptorSet?
        // by setting descriptor_count
        // what will be with dst_binding?
        // how we access in shader?
        let write_desc: Vec<vk::WriteDescriptorSet> = pipe_type.buffers.iter().enumerate().map(
            |(i, _)| vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: ptr::null(),
                dst_set: desc_set[0],
                dst_binding: i as u32,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                p_image_info: ptr::null(),
                p_buffer_info: &buffer_descs[i],
                p_texel_buffer_view: ptr::null()
            }
        ).collect();

        unsafe { pipe_type.device.device().update_descriptor_sets(&write_desc, &[]) };

        let pipeline_cache_info = vk::PipelineCacheCreateInfo {
            s_type: vk::StructureType::PIPELINE_CACHE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineCacheCreateFlags::empty(),
            initial_data_size: 0,
            p_initial_data: ptr::null()
        };

        let pipeline_cache = on_error_ret!(
            unsafe { pipe_type.device.device().create_pipeline_cache(&pipeline_cache_info, None) },
            PipelineError::PipelineCache
        );

        let pipeline_shader = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::COMPUTE,
            module: pipe_type.shader.module(),
            p_name: pipe_type.shader.entry().as_ptr(),
            p_specialization_info: ptr::null()
        };

        let pipeline_info = vk::ComputePipelineCreateInfo {
            s_type: vk::StructureType::COMPUTE_PIPELINE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineCreateFlags::empty(),
            stage: pipeline_shader,
            layout: pipeline_layout,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: 0
        };

        let pipelines = on_error_ret!(
            unsafe { pipe_type.device.device().create_compute_pipelines(pipeline_cache, &[pipeline_info], None) },
            PipelineError::Pipeline
        );

        Ok(
            Pipeline {
                i_dev: pipe_type.device,
                i_pipeline_layout: pipeline_layout,
                i_desc_set_layout: desc_set_layout,
                i_desc_set: desc_set[0],
                i_desc_pool: desc_pool,
                i_pipeline: pipelines[0],
                i_pipeline_cache: pipeline_cache,
            }
        )
    }

    #[doc(hidden)]
    pub fn descriptor_set(&self) -> vk::DescriptorSet {
        self.i_desc_set
    }

    #[doc(hidden)]
    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.i_pipeline_layout
    }

    #[doc(hidden)]
    pub fn pipeline(&self) -> vk::Pipeline {
        self.i_pipeline
    }
}

impl<'a> Drop for Pipeline<'a> {
    fn drop(&mut self) {
        let device = self.i_dev.device();

        unsafe {
            device.destroy_pipeline_layout(self.i_pipeline_layout, None);
            device.destroy_descriptor_set_layout(self.i_desc_set_layout, None);
            device.destroy_descriptor_pool(self.i_desc_pool, None);
            device.destroy_pipeline(self.i_pipeline, None);
            device.destroy_pipeline_cache(self.i_pipeline_cache, None);
        }
    }
}