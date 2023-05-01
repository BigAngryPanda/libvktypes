//! Represent pipeline and its configuration

use ash::vk;

use crate::dev;
use crate::memory;
use crate::shader;

use crate::{on_error, on_error_ret};

use std::sync::Arc;
use std::{fmt, ptr};
use std::error::Error;

/// Note: only [memory](crate::memory::Memory) with memory::UsageFlags::STORAGE_BUFFER is allowed
pub struct PipelineCfg<'a, 'b : 'a> {
    pub buffers: &'a [memory::View<'b>],
    pub shader: &'a shader::Shader,
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

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = match self {
            PipelineError::DescriptorPool => {
                "Failed to create descriptor pool (vkCreateDescriptorPool call failed)"
            },
            PipelineError::DescriptorSetLayout => {
                "Failed to create descriptor set layout (vkCreateDescriptorSetLayout call failed)"
            },
            PipelineError::DescriptorSet => {
                "Failed to allocate descriptor set (vkAllocateDescriptorSets call failed)"
            },
            PipelineError::PipelineLayout => {
                "Failed to create pipeline layout (vkCreatePipelineLayout call failed)"
            },
            PipelineError::PipelineCache => {
                "Failed to create pipeline cache (vkCreatePipelineCache call failed)"
            },
            PipelineError::Pipeline => {
                "Failed to create pipeline (vkCreatePipeline call failed)"
            }
        };

        write!(f, "{:?}", err_msg)
    }
}

impl Error for PipelineError {}

/// Represents compute pipeline
pub struct Pipeline {
    i_core:            Arc<dev::Core>,
    i_pipeline_layout: vk::PipelineLayout,
    i_desc_set_layout: vk::DescriptorSetLayout,
    i_desc_set:        vk::DescriptorSet,
    i_desc_pool:       vk::DescriptorPool,
    i_pipeline:        vk::Pipeline,
    i_pipeline_cache:  vk::PipelineCache,
}

// TODO provide dynamic buffer binding
// TODO shader module must outlive pipeline?
impl Pipeline {
    pub fn new(device: &dev::Device, pipe_type: &PipelineCfg) -> Result<Pipeline, PipelineError> {
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
            unsafe { device.device().create_descriptor_pool(&desc_info, device.allocator()) },
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

        let desc_set_layout = unsafe { on_error!(
            device.device().create_descriptor_set_layout(&desc_layout_info, device.allocator()),
            {
                device.device().destroy_descriptor_pool(desc_pool, device.allocator());
                return Err(PipelineError::DescriptorSetLayout);
            }
        )};

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

        let pipeline_layout = unsafe { on_error!(
            device.device().create_pipeline_layout(&pipeline_layout_info, device.allocator()),
            {
                device.device().destroy_descriptor_set_layout(desc_set_layout, device.allocator());
                device.device().destroy_descriptor_pool(desc_pool, device.allocator());
                return Err(PipelineError::PipelineLayout);
            }
        )};

        let alloc_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_pool: desc_pool,
            descriptor_set_count: 1,
            p_set_layouts: &desc_set_layout
        };

        let desc_set = unsafe { on_error!(
            device.device().allocate_descriptor_sets(&alloc_info),
            {
                device.device().destroy_pipeline_layout(pipeline_layout, device.allocator());
                device.device().destroy_descriptor_set_layout(desc_set_layout, device.allocator());
                device.device().destroy_descriptor_pool(desc_pool, device.allocator());
                return Err(PipelineError::DescriptorSet);
            }
        )};

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

        unsafe { device.device().update_descriptor_sets(&write_desc, &[]) };

        let pipeline_cache_info = vk::PipelineCacheCreateInfo {
            s_type: vk::StructureType::PIPELINE_CACHE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineCacheCreateFlags::empty(),
            initial_data_size: 0,
            p_initial_data: ptr::null()
        };

        let pipeline_cache = unsafe { on_error!(
            device.device().create_pipeline_cache(&pipeline_cache_info, device.allocator()),
            {
                device.device().destroy_pipeline_layout(pipeline_layout, device.allocator());
                device.device().destroy_descriptor_set_layout(desc_set_layout, device.allocator());
                device.device().destroy_descriptor_pool(desc_pool, device.allocator());
                return Err(PipelineError::PipelineCache);
            }
        )};

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

        let pipelines = unsafe { on_error!(
            device.device().create_compute_pipelines(pipeline_cache, &[pipeline_info], device.allocator()),
            {
                device.device().destroy_pipeline_cache(pipeline_cache, device.allocator());
                device.device().destroy_pipeline_layout(pipeline_layout, device.allocator());
                device.device().destroy_descriptor_set_layout(desc_set_layout, device.allocator());
                device.device().destroy_descriptor_pool(desc_pool, device.allocator());
                return Err(PipelineError::Pipeline);
            }
        )};

        Ok(
            Pipeline {
                i_core: device.core().clone(),
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

impl Drop for Pipeline {
    fn drop(&mut self) {
        let device = self.i_core.device();
        let alloc = self.i_core.allocator();

        unsafe {
            device.destroy_pipeline(self.i_pipeline, alloc);
            device.destroy_pipeline_cache(self.i_pipeline_cache, alloc);
            device.destroy_pipeline_layout(self.i_pipeline_layout, alloc);
            device.destroy_descriptor_set_layout(self.i_desc_set_layout, alloc);
            device.destroy_descriptor_pool(self.i_desc_pool, alloc);
        }
    }
}