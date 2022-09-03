//! Graphics pipeline and render pass
//!
//! # RenderPass
//! [`RenderPass`] represents context within graphics pipeline is executed
//!
//! It is defined by 3 components:
//! 1) [subpasses](SubpassInfo)
//! 2) [synchronization between subpasses](SubpassSync)
//! 3) [attachments](AttachmentInfo) which defines what for *all* images are used for

use ash::vk;

use crate::{
    dev,
    surface,
    data_ptr,
    on_error_ret,
    shader
};

use std::ptr;
use std::fmt;
use std::error::Error;
use std::convert::Into;

/// Specify how contents of an attachment are treated at the beginning of a subpass
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.AttachmentLoadOp.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkAttachmentLoadOp.html>"]
pub type AttachmentLoadOp = vk::AttachmentLoadOp;

/// Specify how contents of an attachment are treated at the end of a subpass
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.AttachmentStoreOp.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkAttachmentStoreOp.html>"]
pub type AttachmentStoreOp = vk::AttachmentStoreOp;

/// Layout of image and image subresources
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.ImageLayout.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkImageLayout.html>"]
pub type ImageLayout = vk::ImageLayout;

/// Structure specifying an attachment description
#[derive(Debug)]
pub struct AttachmentInfo {
    pub format: surface::ImageFormat,
    pub load_op: AttachmentLoadOp,
    pub store_op: AttachmentStoreOp,
    pub stencil_load_op: AttachmentLoadOp,
    pub stencil_store_op: AttachmentStoreOp,
    pub initial_layout: ImageLayout,
    pub final_layout: ImageLayout,
}

impl Default for AttachmentInfo {
    fn default() -> Self {
        AttachmentInfo {
            format: surface::ImageFormat::UNDEFINED,
            load_op: AttachmentLoadOp::DONT_CARE,
            store_op: AttachmentStoreOp::DONT_CARE,
            stencil_load_op: AttachmentLoadOp::DONT_CARE,
            stencil_store_op: AttachmentStoreOp::DONT_CARE,
            initial_layout: ImageLayout::PRESENT_SRC_KHR,
            final_layout: ImageLayout::PRESENT_SRC_KHR,
        }
    }
}

#[doc(hidden)]
impl From<&AttachmentInfo> for vk::AttachmentDescription {
    fn from(info: &AttachmentInfo) -> vk::AttachmentDescription {
        vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: info.format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: info.load_op,
            store_op: info.store_op,
            stencil_load_op: info.stencil_load_op,
            stencil_store_op: info.stencil_store_op,
            initial_layout: info.initial_layout,
            final_layout: info.final_layout,
        }
    }
}

/// Pipeline stages
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.PipelineStageFlags.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkPipelineStageFlagBits.html>"]
pub type PipelineStage = vk::PipelineStageFlags;

/// Bitmask specifying memory access types that will participate in a memory dependency
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.AccessFlags.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkAccessFlagBits.html>"]
pub type AccessFlags = vk::AccessFlags;

pub const SUBPASS_EXTERNAL: u32 = vk::SUBPASS_EXTERNAL;

pub const NO_ATTACHMENT: u32 =  vk::ATTACHMENT_UNUSED;

/// Essentially SubpassSync acts like a memory barrier between two (previous and next) subpasses
#[derive(Debug)]
pub struct SubpassSync {
    /// Index of previous subpass in [`RenderPassType::subpasses`] or [`SUBPASS_EXTERNAL`]
    pub src_subpass: u32,
    /// Index of next subpass in [`RenderPassType::subpasses`] or [`SUBPASS_EXTERNAL`]
    pub dst_subpass: u32,
    /// Pipeline stage during which a given attachment was used before
    pub src_stage: PipelineStage,
    /// Pipeline stage during which a given attachment will be used later
    pub dst_stage: PipelineStage,
    /// Types of memory operations that occurred in a src subpass or before a render pass
    pub src_access: AccessFlags,
    /// Types of memory operations that occurred in a dst subpass or after a render pass
    pub dst_access: AccessFlags,
}

#[doc(hidden)]
impl From<&SubpassSync> for vk::SubpassDependency {
    fn from(sync: &SubpassSync) -> vk::SubpassDependency {
        vk::SubpassDependency {
            src_subpass: sync.src_subpass,
            dst_subpass: sync.dst_subpass,
            src_stage_mask: sync.src_stage,
            dst_stage_mask: sync.dst_stage,
            src_access_mask: sync.src_access,
            dst_access_mask: sync.dst_access,
            dependency_flags: vk::DependencyFlags::BY_REGION,
        }
    }
}

#[derive(Debug)]
struct SubpassView {
    pub depth_attachment: vk::AttachmentReference,
    pub resolve_attachment: Vec<vk::AttachmentReference>,
    pub color_attachment: Vec<vk::AttachmentReference>,
    pub input_attachment: Vec<vk::AttachmentReference>,
    pub preserve_attachments: Vec<u32>,
}

#[doc(hidden)]
impl From<&SubpassView> for vk::SubpassDescription {
    fn from(view: &SubpassView) -> Self {
        vk::SubpassDescription {
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: view.input_attachment.len() as u32,
            p_input_attachments: data_ptr!(view.input_attachment),
            color_attachment_count: view.color_attachment.len() as u32,
            p_color_attachments: data_ptr!(view.color_attachment),
            p_resolve_attachments: data_ptr!(view.resolve_attachment),
            p_depth_stencil_attachment: &view.depth_attachment,
            preserve_attachment_count: view.preserve_attachments.len() as u32,
            p_preserve_attachments: data_ptr!(view.preserve_attachments),
        }
    }
}

/// `Subpass` configuration
///
/// All information about [valid usage](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkSubpassDescription.html)
///
/// Note: [`SubpassInfo::resolve_attachments`] **must be** `&[]` or same length as [`SubpassInfo::color_attachments`]
#[derive(Debug)]
pub struct SubpassInfo<'a> {
    pub input_attachments: &'a [u32],
    pub color_attachments: &'a [u32],
    pub resolve_attachments: &'a [u32],
    pub depth_stencil_attachment: u32,
    pub preserve_attachments: &'a [u32],
}

impl<'a> Default for SubpassInfo<'a> {
    fn default() -> SubpassInfo<'a> {
        SubpassInfo {
            input_attachments: &[],
            color_attachments: &[],
            resolve_attachments: &[],
            depth_stencil_attachment: NO_ATTACHMENT,
            preserve_attachments: &[],
        }
    }
}

#[doc(hidden)]
impl From<&SubpassInfo<'_>> for SubpassView {
    fn from(info: &SubpassInfo) -> Self {
        let input_attch: Vec<vk::AttachmentReference> = info
            .input_attachments
            .iter()
            .map(|&i| vk::AttachmentReference {
                attachment: i,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            })
            .collect();

        let color_attch: Vec<vk::AttachmentReference> = info
            .color_attachments
            .iter()
            .map(|&i| vk::AttachmentReference {
                attachment: i,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            })
            .collect();

        let resolve_attch: Vec<vk::AttachmentReference> = info
            .resolve_attachments
            .iter()
            .map(|&i| vk::AttachmentReference {
                attachment: i,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            })
            .collect();

        let depth_attch = vk::AttachmentReference {
            attachment: info.depth_stencil_attachment,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        SubpassView {
            depth_attachment: depth_attch,
            resolve_attachment: resolve_attch,
            color_attachment: color_attch,
            input_attachment: input_attch,
            preserve_attachments: info.preserve_attachments.to_vec(),
        }
    }
}

#[derive(Debug)]
pub enum RenderPassError {
    /// Error was returned as a result of `vkCreateRenderPass`
    /// [call](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateRenderPass.html)
    Creation,
}

impl fmt::Display for RenderPassError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vkCreateRenderPass call failed")
    }
}

impl Error for RenderPassError { }

/// [`RenderPass`] configuration
pub struct RenderPassType<'a, 'b: 'a> {
    pub device: &'b dev::Device,
    pub attachments: &'a [AttachmentInfo],
    pub sync_info: &'a [SubpassSync],
    pub subpasses: &'a [SubpassInfo<'b>],
}

/// Context for executing graphics pipeline
pub struct RenderPass<'a> {
    i_dev: &'a dev::Device,
    i_rp: vk::RenderPass,
}

impl<'a> RenderPass<'a> {
    pub fn new(rp_type: &'a RenderPassType) -> Result<RenderPass<'a>, RenderPassError> {
        let dependencies: Vec<vk::SubpassDependency> = rp_type
            .sync_info
            .iter()
            .map(|x| x.into())
            .collect();

        let attachments: Vec<vk::AttachmentDescription> = rp_type
            .attachments
            .iter()
            .map(|x| x.into())
            .collect();

        let subpasses_slice: Vec<SubpassView> = rp_type
            .subpasses
            .iter()
            .map(|x| x.into())
            .collect();

        let subpasses: Vec<vk::SubpassDescription> = subpasses_slice
            .iter()
            .map(|x| x.into())
            .collect();

        let render_pass_create_info:vk::RenderPassCreateInfo = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::RenderPassCreateFlags::empty(),
            attachment_count: attachments.len() as u32,
            p_attachments: data_ptr!(attachments),
            subpass_count: subpasses.len() as u32,
            p_subpasses: data_ptr!(subpasses),
            dependency_count: dependencies.len() as u32,
            p_dependencies: data_ptr!(dependencies),
        };

        let rp = on_error_ret!(
            unsafe { rp_type.device.device().create_render_pass(&render_pass_create_info, None) },
            RenderPassError::Creation
        );

        Ok(
            RenderPass {
                i_dev: rp_type.device,
                i_rp: rp,
            }
        )
    }

    /// Create [`RenderPass`] with single subpass and single attachment
    pub fn single_subpass(dev: &'a dev::Device, img_format: surface::ImageFormat)
        -> Result<RenderPass<'a>, RenderPassError>
    {
        let dependencies:[vk::SubpassDependency; 2] = [
            vk::SubpassDependency {
                src_subpass: vk::SUBPASS_EXTERNAL,
                dst_subpass: 0,
                src_stage_mask: vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                src_access_mask: vk::AccessFlags::MEMORY_READ,
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dependency_flags: vk::DependencyFlags::BY_REGION,
            },
            vk::SubpassDependency {
                src_subpass: 0,
                dst_subpass: vk::SUBPASS_EXTERNAL,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_stage_mask: vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_access_mask: vk::AccessFlags::MEMORY_READ,
                dependency_flags: vk::DependencyFlags::BY_REGION,
            }
        ];

        let attachment_descriptions:[vk::AttachmentDescription; 1] = [
            vk::AttachmentDescription {
                flags: vk::AttachmentDescriptionFlags::empty(),
                format: img_format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            }
        ];

        let color_attachment_references:[vk::AttachmentReference; 1] = [
            vk::AttachmentReference {
                attachment: 0,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            }
        ];

        let subpass_descriptions:[vk::SubpassDescription; 1] = [
            vk::SubpassDescription {
                flags: vk::SubpassDescriptionFlags::empty(),
                pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
                input_attachment_count: 0,
                p_input_attachments: ptr::null(),
                color_attachment_count: 1,
                p_color_attachments: &color_attachment_references[0],
                p_resolve_attachments: ptr::null(),
                p_depth_stencil_attachment: ptr::null(),
                preserve_attachment_count: 0,
                p_preserve_attachments: ptr::null(),
            }
        ];

        let render_pass_create_info:vk::RenderPassCreateInfo = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::RenderPassCreateFlags::empty(),
            attachment_count: 1,
            p_attachments: &attachment_descriptions[0],
            subpass_count: 1,
            p_subpasses: &subpass_descriptions[0],
            dependency_count: 2,
            p_dependencies: &dependencies[0],
        };

        let rp = on_error_ret!(
            unsafe { dev.device().create_render_pass(&render_pass_create_info, None) },
            RenderPassError::Creation
        );

        Ok(
            RenderPass {
                i_dev: dev,
                i_rp: rp,
            }
        )
    }

    #[doc(hidden)]
    fn render_pass(&self) -> vk::RenderPass {
        self.i_rp
    }
}

impl<'a> Drop for RenderPass<'a> {
    fn drop(&mut self) {
        unsafe {
            self.i_dev.device().destroy_render_pass(self.i_rp, None);
        }
    }
}

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
/// use libvktypes::surface::ImageFormat;
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
    pub format: surface::ImageFormat,
    /// Beginning of data for a given attribute
    pub offset: u32,
}

impl Default for VertexInputCfg {
    fn default() -> VertexInputCfg {
        VertexInputCfg {
            location: 0,
            binding: 0,
            format: surface::ImageFormat::UNDEFINED,
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
    pub device: &'a dev::Device,
    pub vertex_shader: &'a shader::Shader<'a>,
    /// Size of every vertex
    pub vertex_size: u32,
    /// Number of vertex binding slots
    pub vert_slots: u32,
    pub vert_input: &'a [VertexInputCfg],
    pub frag_shader: &'a shader::Shader<'a>,
    pub topology: Topology,
    pub extent: surface::Extent2D,
    pub push_constant_size: u32,
    pub render_pass: &'a RenderPass<'a>,
    /// Subpass index inside [`RenderPass`](PipelineType::render_pass)
    pub subpass_index: u32,
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

pub struct Pipeline<'a> {
    i_dev: &'a dev::Device,
    i_layout: vk::PipelineLayout,
    i_pipeline: vk::Pipeline,
}

impl<'a> Pipeline<'a> {
    pub fn new(pipe_cfg: &'a PipelineType) -> Result<Pipeline<'a>, PipelineError> {
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
            (0..pipe_cfg.vert_slots)
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
		    unsafe { pipe_cfg.device.device().create_pipeline_layout(&layout_create_info, None) },
            PipelineError::Layout
        );

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
            p_depth_stencil_state: ptr::null(),
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
                pipe_cfg
                .device
                .device()
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[pipeline_create_info],
                    None
                )
            },
            PipelineError::Pipeline
        );


        Ok(
            Pipeline {
                i_dev: pipe_cfg.device,
                i_layout: pipeline_layout,
                i_pipeline: pipeline[0],
            }
        )
    }
}

impl<'a> Drop for Pipeline<'a> {
    fn drop(&mut self) {
        unsafe {
            self.i_dev.device().destroy_pipeline_layout(self.i_layout, None);
            self.i_dev.device().destroy_pipeline(self.i_pipeline, None);
        }
    }
}