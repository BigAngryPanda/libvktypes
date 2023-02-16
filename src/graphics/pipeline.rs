//! Pipeline configuration

use ash::vk;

use crate::{
    dev,
    surface,
    memory,
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

pub struct PipelineType<'a> {
    pub vertex_shader: &'a shader::Shader,
    /// Size of every vertex
    pub vertex_size: u32,
    pub vert_input: &'a [VertexInputCfg],
    pub frag_shader: &'a shader::Shader,
    pub topology: Topology,
    pub extent: surface::Extent2D,
    pub push_constant_size: u32,
    pub render_pass: &'a graphics::RenderPass,
    /// Subpass index inside [`RenderPass`](PipelineType::render_pass)
    pub subpass_index: u32,
    pub enable_depth: bool,
}

#[derive(Debug)]
pub enum PipelineError {
    /// Failed to create pipeline layout
    Layout,
    /// Failed to create pipeline
    Pipeline
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipelineError::Layout => write!(f, "vkCreatePipelineLayout call failed"),
            PipelineError::Pipeline => write!(f, "vkCreateGraphicsPipelines call failed"),
        }
    }
}

impl Error for PipelineError { }

pub struct Pipeline {
    i_core: Arc<dev::Core>,
    i_layout: vk::PipelineLayout,
    i_pipeline: vk::Pipeline,
}

impl Pipeline {
    pub fn new(device: &dev::Device, pipe_cfg: &PipelineType) -> Result<Pipeline, PipelineError> {
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                stage: vk::ShaderStageFlags::VERTEX,
                module: pipe_cfg.vertex_shader.module(),
                p_name: pipe_cfg.frag_shader.entry().as_ptr(),
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

        let input_assembly_state_create_info = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
            topology: pipe_cfg.topology,
            primitive_restart_enable: ash::vk::FALSE,
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
        let layout_create_info:vk::PipelineLayoutCreateInfo = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: 0,
            p_set_layouts: ptr::null(),
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

        let pipeline_layout = on_error_ret!(
		    unsafe { device.device().create_pipeline_layout(&layout_create_info, device.allocator()) },
            PipelineError::Layout
        );

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

        let pipeline_create_info:vk::GraphicsPipelineCreateInfo = vk::GraphicsPipelineCreateInfo {
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
            p_depth_stencil_state: if pipe_cfg.enable_depth {
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

        let pipeline = on_error_ret!(
            unsafe {
                device
                .device()
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[pipeline_create_info],
                    device.allocator()
                )
            },
            PipelineError::Pipeline
        );


        Ok(
            Pipeline {
                i_core: device.core().clone(),
                i_layout: pipeline_layout,
                i_pipeline: pipeline[0],
            }
        )
    }

    #[doc(hidden)]
    pub fn pipeline(&self) -> vk::Pipeline {
        self.i_pipeline
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.i_core.device().destroy_pipeline_layout(self.i_layout, self.i_core.allocator());
            self.i_core.device().destroy_pipeline(self.i_pipeline, self.i_core.allocator());
        }
    }
}