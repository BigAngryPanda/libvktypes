//! Pipeline configuration

use ash::vk;
use ash::prelude::VkResult;

use crate::{
    dev,
    memory,
    on_error,
    data_ptr,
    on_error_ret,
    shader,
    graphics
};

use std::ptr;
use std::fmt;
use std::sync::Arc;
use std::error::Error;

/// Configuration of pipeline's vertex stage input
///
/// Example
///
/// ```ignore
///     // part of vertex shader code
///     layout(location = 0) in vec4 Position;
///     layout(location = 1) in vec4 Color;
///
///     // ...
/// ```
/// And corresponding configuration
/// ```
/// // Vertex
/// use libvktypes::memory::ImageFormat;
/// use libvktypes::graphics::VertexInputCfg;
///
/// struct Vertex {
///     pos: [f32; 4],
///     color: [f32; 4],
/// }
///
/// let cfg = [
///     // Position
///     VertexInputCfg {
///         location: 0,
///         binding: 0,
///         format: ImageFormat::R32G32B32A32_SFLOAT,
///         offset: 0,
///     },
///     // Color
///     VertexInputCfg {
///         location: 1,
///         binding: 0,
///         format: ImageFormat::R32G32B32A32_SFLOAT,
///         offset: std::mem::size_of::<[f32; 4]>() as u32,
///     }
/// ];
///
/// ```
#[derive(Debug, Clone, Copy)]
pub struct VertexInputCfg {
    /// Index of an attribute, the same as defined by the location layout specifier in a shader source code
    pub location: u32,
    /// The number of the slot from which data should be read
    pub binding: u32,
    /// Data type and number of components per attribute
    pub format: memory::ImageFormat,
    /// Beginning of data for a given attribute
    pub offset: u32,
}

impl Default for VertexInputCfg {
    fn default() -> VertexInputCfg {
        VertexInputCfg {
            location: 0,
            binding: 0,
            format: memory::ImageFormat::UNDEFINED,
            offset: 0,
        }
    }
}

#[doc(hidden)]
impl From<&VertexInputCfg> for vk::VertexInputAttributeDescription {
    fn from(cfg: &VertexInputCfg) -> Self {
        vk::VertexInputAttributeDescription {
            location: cfg.location,
            binding: cfg.binding,
            format: cfg.format,
            offset: cfg.offset,
        }
    }
}

/// Describe how vertices should be assembled into primitives
///
#[doc = "Possible values: <https://docs.rs/ash/latest/ash/vk/struct.PrimitiveTopology.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkPrimitiveTopology.html>"]
pub type Topology = vk::PrimitiveTopology;

/// Pipeline configuration
///
/// # Vertex stage configuration
/// [`vertex_shader`](PipelineCfg::vertex_shader) is your vertex shader module (pretty straightforward)
///
/// `vertex_size` is the size of every vertex
///
/// For example you may pass just vertex coordinates in 3D space as `[f32; 3]` (vertex_size will be `size_of::<[f32; 3]>()`)
/// or with 4-th coordinate as `[f32; 4]` (vertex_size will be `size_of::<[f32; 4]>()`)
///
/// Do not confuse with [`VertexInputCfg`]
///
/// `vert_input` has its own documentation [here](VertexInputCfg)
///
/// Vertices must be in counterclockwise order
///
/// # Topology
/// A good explanation about topologies may be found
/// [here](https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#drawing-primitive-topology-class)
///
/// # Depth test
/// Set [`enable_depth_test`](PipelineCfg::enable_depth_test) to perform depth test
///
/// However you have to allocate depth buffer and properly pass it to the render pass
///
/// # Shaders and sets
/// `sets` represents shader layout description
///
/// In other words `sets[X][Y]` corresponds to
/// `layout(set=X, binding=Y) ...`
///
/// If you want to "skip" some sets leave empty array for corresponding sets
///
/// If you want to "skip" some bindings use [`Resource::empty()`](crate::graphics::Resource::empty())
///
/// More detailed information can be found in `Resource` [documentation](crate::graphics::Resource)
///
/// # Assembly restarting
/// Affects [indexed drawing](crate::cmd::Buffer::draw_indexed)
///
/// `enable_depth_test` controls whether a special vertex index value is treated as restarting the assembly of primitives
///
/// For example the special index value is
/// [`INDEX_REASSEMBLY_UINT32`](memory::INDEX_REASSEMBLY_UINT32) for `IndexBufferType::UINT32`
/// and so on
///
/// Read more [here](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkPipelineInputAssemblyStateCreateInfo.html)
pub struct PipelineCfg<'a, 'b : 'a> {
    pub vertex_shader: &'a shader::Shader,
    /// Size of every vertex
    pub vertex_size: u32,
    pub vert_input: &'a [VertexInputCfg],
    pub frag_shader: &'a shader::Shader,
    pub topology: Topology,
    pub extent: memory::Extent2D,
    pub push_constant_size: u32,
    pub render_pass: &'a graphics::RenderPass,
    /// Subpass index inside [`RenderPass`](PipelineCfg::render_pass)
    pub subpass_index: u32,
    pub enable_depth_test: bool,
    pub enable_primitive_restart: bool,
    pub sets: &'a [&'a [&'a graphics::Resource<'b>]]
}

#[derive(Debug)]
pub enum PipelineError {
    DescriptorPool,
    DescriptorSet,
    DescriptorAllocation,
    /// Failed to create pipeline layout
    Layout,
    /// Failed to create pipeline
    Pipeline
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipelineError::DescriptorPool => write!(f, "Failed to create descriptor pool (vkCreateDescriptorPool call failed)"),
            PipelineError::DescriptorSet => write!(f, "Failed to create descriptor set layout (vkCreateDescriptorSetLayout call failed)"),
            PipelineError::DescriptorAllocation => write!(f, "Failed to allocate descriptor set (vkDescriptorSetAllocateInfo call failed)"),
            PipelineError::Layout => write!(f, "vkCreatePipelineLayout call failed"),
            PipelineError::Pipeline => write!(f, "vkCreateGraphicsPipelines call failed"),
        }
    }
}

impl Error for PipelineError { }

/// Graphics pipeline
pub struct Pipeline {
    i_core: Arc<dev::Core>,
    i_layout: vk::PipelineLayout,
    i_pipeline: vk::Pipeline,
    i_desc_pool: vk::DescriptorPool,
    i_desc_sets: Vec<vk::DescriptorSet>,
    i_desc_layouts: Vec<vk::DescriptorSetLayout>
}

impl Pipeline {
    pub fn new(device: &dev::Device, pipe_cfg: &PipelineCfg) -> Result<Pipeline, PipelineError> {
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                stage: vk::ShaderStageFlags::VERTEX,
                module: pipe_cfg.vertex_shader.module(),
                p_name: pipe_cfg.vertex_shader.entry().as_ptr(),
                p_specialization_info: ptr::null(),
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                module: pipe_cfg.frag_shader.module(),
                p_name: pipe_cfg.frag_shader.entry().as_ptr(),
                p_specialization_info: ptr::null(),
            },
        ];

        let vertex_binding_descriptions: Vec<vk::VertexInputBindingDescription> =
            (0..pipe_cfg.vert_input.len() as u32)
            .map(|i| vk::VertexInputBindingDescription {
                binding: i,
                stride: pipe_cfg.vertex_size,
                input_rate: vk::VertexInputRate::VERTEX,
            })
            .collect();

        let vertex_attribute_descriptions: Vec<vk::VertexInputAttributeDescription> =
            pipe_cfg.vert_input.iter().map(|x| x.into()).collect();

        let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineVertexInputStateCreateFlags::empty(),
            vertex_binding_description_count: vertex_binding_descriptions.len() as u32,
            p_vertex_binding_descriptions: data_ptr!(vertex_binding_descriptions),
            vertex_attribute_description_count: vertex_attribute_descriptions.len() as u32,
            p_vertex_attribute_descriptions: data_ptr!(vertex_attribute_descriptions),
        };

        let desc_pool = if !pipe_cfg.sets.is_empty() {
            on_error_ret!(
                create_descriptor_pool(device, pipe_cfg.sets),
                PipelineError::DescriptorPool
            )
        } else {
            vk::DescriptorPool::null()
        };

        let mut sets_layout: Vec<vk::DescriptorSetLayout> = Vec::new();

        if !pipe_cfg.sets.is_empty() {
            for res in pipe_cfg.sets {
                match create_set_layout(device, res) {
                    Ok(set) => sets_layout.push(set),
                    Err(_) => {
                        clear_sets_layout(device, &sets_layout, desc_pool);
                        return Err(PipelineError::DescriptorSet);
                    }
                }
            }
        }

        let sets = if !pipe_cfg.sets.is_empty() {
            on_error!(
                allocate_descriptor_sets(device, &sets_layout, desc_pool),
                {
                    clear_sets_layout(device, &sets_layout, desc_pool);
                    return Err(PipelineError::DescriptorAllocation);
                }
            )
        } else {
            Vec::new()
        };

        let input_assembly_state_create_info = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
            topology: pipe_cfg.topology,
            primitive_restart_enable: pipe_cfg.enable_primitive_restart as ash::vk::Bool32,
        };

        let viewports = [vk::Viewport {
            x: 0_f32,
            y: 0_f32,
            width: pipe_cfg.extent.width as f32,
            height: pipe_cfg.extent.height as f32,
            min_depth: 0_f32,
            max_depth: 1_f32,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: pipe_cfg.extent,
        }];

        /*
            Now we must specify the form of output data
            Viewport specifies to what part of the image (or texture, or window) we want do draw
        */
        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineViewportStateCreateFlags::empty(),
            viewport_count: viewports.len() as u32,
            p_viewports: data_ptr!(viewports),
            scissor_count: scissors.len() as u32,
            p_scissors: data_ptr!(scissors),
        };

        /*
            The next part of the graphics pipeline creation applies to the rasterization state
            We must specify how polygons are going to be rasterized (changed into fragments)
        */
        let rasterization_state_create_info = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable: ash::vk::FALSE,
            rasterizer_discard_enable: ash::vk::FALSE,
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            depth_bias_enable: ash::vk::FALSE,
            depth_bias_constant_factor: 0.0,
            depth_bias_clamp: 0.0,
            depth_bias_slope_factor: 0.0,
            line_width: 1.0,
        };

        /*
            In Vulkan, when we are creating a graphics pipeline, we must also specify the state relevant to multisampling
        */
        let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineMultisampleStateCreateFlags::empty(),
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            sample_shading_enable: ash::vk::FALSE,
            min_sample_shading: 1.0,
            p_sample_mask: ptr::null(),
            alpha_to_coverage_enable: ash::vk::FALSE,
            alpha_to_one_enable: ash::vk::FALSE,
        };

        let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState {
            blend_enable: ash::vk::FALSE,
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        };

        let color_blend_state_create_info = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable: ash::vk::FALSE,
            logic_op: vk::LogicOp::COPY,
            attachment_count: 1,
            p_attachments: &color_blend_attachment_state,
            blend_constants: [0.0; 4],
        };

        let push_const_range = vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::ALL_GRAPHICS,
            offset: 0,
            size: pipe_cfg.push_constant_size,
        };

        /*
            A pipeline layout describes all the resources that can be accessed by the pipeline
        */
        let layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: sets_layout.len() as u32,
            p_set_layouts: data_ptr!(sets_layout),
            push_constant_range_count: if pipe_cfg.push_constant_size != 0 {
                1
            } else {
                0
            },
            p_push_constant_ranges: if pipe_cfg.push_constant_size != 0 {
                &push_const_range
            } else {
                ptr::null()
            },
        };

        let pipeline_layout = unsafe { on_error!(
		    device.device().create_pipeline_layout(&layout_create_info, device.allocator()),
            {
                clear_sets_layout(device, &sets_layout, desc_pool);
                return Err(PipelineError::Layout);
            }
        )};

        let depth_cfg = vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
            depth_bounds_test_enable: 0,
            stencil_test_enable: 0,
            front: vk::StencilOpState::default(),
            back: vk::StencilOpState::default(),
            min_depth_bounds: f32::default(),
            max_depth_bounds: f32::default(),
        };

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineCreateFlags::empty(),
            stage_count: shader_stage_create_infos.len() as u32,
            p_stages: shader_stage_create_infos.as_ptr(),
            p_vertex_input_state: &vertex_input_state_create_info,
            p_input_assembly_state: &input_assembly_state_create_info,
            p_tessellation_state: ptr::null(),
            p_viewport_state: &viewport_state_create_info,
            p_rasterization_state: &rasterization_state_create_info,
            p_multisample_state: &multisample_state_create_info,
            p_depth_stencil_state: if pipe_cfg.enable_depth_test {
                &depth_cfg
            } else {
                ptr::null()
            },
            p_color_blend_state: &color_blend_state_create_info,
            p_dynamic_state: ptr::null(),
            layout: pipeline_layout,
            render_pass: pipe_cfg.render_pass.render_pass(),
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: -1,
        };

        let pipeline = unsafe { on_error!(
            device
            .device()
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[pipeline_create_info],
                device.allocator()
            ),
            {
                clear_sets_layout(device, &sets_layout, desc_pool);
                device.device().destroy_pipeline_layout(pipeline_layout, device.allocator());
                return Err(PipelineError::Pipeline);
            }
        )};


        Ok(
            Pipeline {
                i_core: device.core().clone(),
                i_layout: pipeline_layout,
                i_pipeline: pipeline[0],
                i_desc_pool: desc_pool,
                i_desc_sets: sets,
                i_desc_layouts: sets_layout
            }
        )
    }

    /// # Description
    ///
    /// Specify what resources should use pipeline
    ///
    /// Note: do not confuse with [`bind_resources`](crate::cmd::Buffer::bind_resources)
    ///
    /// The latest method just bind together pipeline with "pointers to resources"
    ///
    /// Which contains in [`Pipeline`] according to [`PipelineCfg::sets`]
    ///
    /// `update` on the other hand overwrite "pointers" to the selected resource
    ///
    /// # Resource compatibility
    ///
    /// Each resource in `data` must be compatiable with corresponding [set](PipelineCfg::sets)
    ///
    /// `data[0][0]` with `sets[0][0]` and so on
    pub fn update(&self, data: &[&[&graphics::Resource]]) {
        for i in 0..self.i_desc_sets.len() {
            self.update_set(data[i], i);
        }
    }

    /// Update selected set
    ///
    /// For requirements see [`Pipeline::update`]
    pub fn update_set(&self, resources: &[&graphics::Resource], set_index: usize) {
        let mut buffer_descs: Vec<vk::DescriptorBufferInfo> = Vec::new();

        for resource in resources {
            for view in resource.views() {
                buffer_descs.push(
                    vk::DescriptorBufferInfo {
                        buffer: view.buffer(),
                        offset: 0,
                        range: vk::WHOLE_SIZE
                    }
                );
            }
        }

        let write_desc: Vec<vk::WriteDescriptorSet> = resources.iter().enumerate().map(
            |(i, res)| vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: ptr::null(),
                dst_set: self.i_desc_sets[set_index],
                dst_binding: i as u32,
                dst_array_element: 0,
                descriptor_count: res.count(),
                descriptor_type: res.resource_type(),
                p_image_info: ptr::null(),
                p_buffer_info: &buffer_descs[i],
                p_texel_buffer_view: ptr::null()
            }
        ).collect();

        unsafe {
            self.i_core.device().update_descriptor_sets(&write_desc, &[])
        };
    }

    #[doc(hidden)]
    pub fn pipeline(&self) -> vk::Pipeline {
        self.i_pipeline
    }

    #[doc(hidden)]
    pub fn layout(&self) -> vk::PipelineLayout {
        self.i_layout
    }

    #[doc(hidden)]
    pub fn descriptor_set(&self) -> &[vk::DescriptorSet] {
        &self.i_desc_sets
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            if self.i_desc_pool != vk::DescriptorPool::null() {
                self
                .i_core
                .device()
                .destroy_descriptor_pool(self.i_desc_pool, self.i_core.allocator());
                for &set in &self.i_desc_layouts {
                    self
                    .i_core
                    .device()
                    .destroy_descriptor_set_layout(set, self.i_core.allocator());
                }
            }
            self.i_core.device().destroy_pipeline_layout(self.i_layout, self.i_core.allocator());
            self.i_core.device().destroy_pipeline(self.i_pipeline, self.i_core.allocator());
        }
    }
}

fn create_descriptor_pool(
    device: &dev::Device,
    resources: &[&[&graphics::Resource]]
) -> VkResult<vk::DescriptorPool> {
    let mut desc_size: Vec<vk::DescriptorPoolSize> = Vec::new();

    for &set in resources {
        for binding in set {
            desc_size.push(vk::DescriptorPoolSize {
                ty: binding.resource_type(),
                descriptor_count: binding.count()
            });
        }
    }

    let desc_info = vk::DescriptorPoolCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: resources.len() as u32,
        pool_size_count: desc_size.len() as u32,
        p_pool_sizes: desc_size.as_ptr(),
    };

    unsafe {
        device.device().create_descriptor_pool(&desc_info, device.allocator())
    }
}

fn create_set_layout(
    device: &dev::Device,
    resources: &[&graphics::Resource]
) -> VkResult<vk::DescriptorSetLayout> {
    let bindings: Vec<vk::DescriptorSetLayoutBinding> = resources.iter().enumerate().map(
        |(i, binding)| vk::DescriptorSetLayoutBinding {
            binding: i as u32,
            descriptor_type: binding.resource_type(),
            descriptor_count: binding.count(),
            stage_flags: binding.stage(),
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

    unsafe {
        device.device().create_descriptor_set_layout(&desc_layout_info, device.allocator())
    }
}

fn clear_sets_layout(
    device: &dev::Device,
    sets: &Vec<vk::DescriptorSetLayout>,
    pool: vk::DescriptorPool)
{
    if pool == vk::DescriptorPool::null() {
        return;
    }

    unsafe {
        device
        .device()
        .destroy_descriptor_pool(pool, device.allocator());

        for &set in sets {
            device
            .device()
            .destroy_descriptor_set_layout(set, device.allocator());
        }
    }
}

fn allocate_descriptor_sets(
    device: &dev::Device,
    sets: &Vec<vk::DescriptorSetLayout>,
    pool: vk::DescriptorPool
) -> VkResult<Vec<vk::DescriptorSet>> {
    let alloc_info = vk::DescriptorSetAllocateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
        p_next: ptr::null(),
        descriptor_pool: pool,
        descriptor_set_count: sets.len() as u32,
        p_set_layouts: sets.as_ptr()
    };

    unsafe {
        device.device().allocate_descriptor_sets(&alloc_info)
    }
}