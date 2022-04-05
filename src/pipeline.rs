//! Represent pipeline and its configuration

use ash::vk;

use crate::logical_device::LogicalDevice;
use crate::memory::{
	Memory,
	BufferDescriptor
};
use crate::shader::Shader;
use crate::specialization_constants::SpecializationConstant;

use crate::on_error;

use std::ptr;
use std::ffi::CString;

#[derive(Debug)]
pub enum ComputePipelineError {
	DescriptorPool,
    DescriptorSetLayout,
	DescriptorSet,
	PipelineLayout,
	PipelineCache,
	Pipeline
}

/// Represents single pipeline
pub struct ComputePipeline<'a> {
	i_ldev:            &'a LogicalDevice<'a>,
	i_pipeline_layout: vk::PipelineLayout,
	i_desc_set_layout: vk::DescriptorSetLayout,
	i_desc_set:        vk::DescriptorSet,
	i_desc_pool:       vk::DescriptorPool,
	i_pipeline:        vk::Pipeline,
	i_pipeline_cache:  vk::PipelineCache,
}

// TODO provide dynamic buffer binding
// TODO shader module must outlive pipeline?
impl<'a> ComputePipeline<'a> {
	pub fn new(dev: &'a LogicalDevice,
				buffers: &[&Memory],
				shader: &Shader,
				spec_data: &SpecializationConstant,
				push_const_size: u32) -> Result<ComputePipeline<'a>, ComputePipelineError> {
		let desc_size:[vk::DescriptorPoolSize; 1] =
		[
			vk::DescriptorPoolSize {
				ty: vk::DescriptorType::STORAGE_BUFFER,
				descriptor_count: buffers.len() as u32,
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

		let desc_pool = on_error!(
			unsafe { dev.i_device.create_descriptor_pool(&desc_info, None) },
			return Err(ComputePipelineError::DescriptorPool)
		);

		let bindings: Vec<vk::DescriptorSetLayoutBinding> = buffers.iter().enumerate().map(
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

		let desc_set_layout = on_error!(
			unsafe { dev.i_device.create_descriptor_set_layout(&desc_layout_info, None) },
			return Err(ComputePipelineError::DescriptorSetLayout)
		);

		let push_const_range = vk::PushConstantRange {
			stage_flags: vk::ShaderStageFlags::COMPUTE,
			offset: 0,
			size: push_const_size,
		};

		let pipeline_layout_info = vk::PipelineLayoutCreateInfo {
			s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
			p_next: ptr::null(),
			flags: vk::PipelineLayoutCreateFlags::empty(),
			set_layout_count: 1,
			p_set_layouts: &desc_set_layout,
			push_constant_range_count: if push_const_size != 0 { 1 } else { 0 },
			p_push_constant_ranges: if push_const_size != 0 { &push_const_range } else { ptr::null() }
		};

		let pipeline_layout = on_error!(
			unsafe { dev.i_device.create_pipeline_layout(&pipeline_layout_info, None) },
			return Err(ComputePipelineError::PipelineLayout)
		);

		let alloc_info = vk::DescriptorSetAllocateInfo {
			s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
			p_next: ptr::null(),
			descriptor_pool: desc_pool,
			descriptor_set_count: 1,
			p_set_layouts: &desc_set_layout
		};

		let desc_set = on_error!(
			unsafe { dev.i_device.allocate_descriptor_sets(&alloc_info) },
			return Err(ComputePipelineError::DescriptorSet)
		);

		let buffer_descs: Vec<BufferDescriptor> = buffers.iter().map(
			|b| b.get_descriptor()
		).collect();

		// TODO big question can we update set with single vk::WriteDescriptorSet?
		// by setting descriptor_count
		// what will be with dst_binding?
		// how we access in shader?
		let write_desc: Vec<vk::WriteDescriptorSet> = buffers.iter().enumerate().map(
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

		unsafe { dev.i_device.update_descriptor_sets(&write_desc, &[]) };

		let pipeline_cache_info = vk::PipelineCacheCreateInfo {
			s_type: vk::StructureType::PIPELINE_CACHE_CREATE_INFO,
			p_next: ptr::null(),
			flags: vk::PipelineCacheCreateFlags::empty(),
			initial_data_size: 0,
			p_initial_data: ptr::null()
		};

		let pipeline_cache = on_error!(
			unsafe { dev.i_device.create_pipeline_cache(&pipeline_cache_info, None) },
			return Err(ComputePipelineError::PipelineCache)
		);

// TODO handle case when we failed CString creation
		let entry_name = CString::new(shader.i_entry.clone()).expect("");

		let pipeline_shader = vk::PipelineShaderStageCreateInfo {
			s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
			p_next: ptr::null(),
			flags: vk::PipelineShaderStageCreateFlags::empty(),
			stage: vk::ShaderStageFlags::COMPUTE,
			module: shader.i_module,
			p_name: entry_name.as_ptr(),
			p_specialization_info: spec_data.info()
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

		let pipelines = on_error!(
			unsafe { dev.i_device.create_compute_pipelines(pipeline_cache, &[pipeline_info], None) },
			return Err(ComputePipelineError::Pipeline)
		);

        Ok(
			ComputePipeline {
				i_ldev           : dev,
				i_pipeline_layout: pipeline_layout,
				i_desc_set_layout: desc_set_layout,
				i_desc_set       : desc_set[0],
				i_desc_pool      : desc_pool,
				i_pipeline       : pipelines[0],
				i_pipeline_cache : pipeline_cache,
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

impl<'a> Drop for ComputePipeline<'a> {
	fn drop(&mut self) {
		let device = self.i_ldev.device();

		unsafe {
			device.destroy_pipeline_layout(self.i_pipeline_layout, None);
			device.destroy_descriptor_set_layout(self.i_desc_set_layout, None);
			device.destroy_descriptor_pool(self.i_desc_pool, None);
			device.destroy_pipeline(self.i_pipeline, None);
			device.destroy_pipeline_cache(self.i_pipeline_cache, None);
		}
	}
}