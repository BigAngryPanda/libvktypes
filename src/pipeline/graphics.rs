use ash::vk;

use crate::{
    dev,
    memory,
    data_ptr,
    on_error_ret,
    shader,
    graphics,
    pipeline
};

use std::sync::Arc;

use core::ffi::c_char;

/// Graphics pipeline configuration
pub struct GraphicsPipelineBuilder {
    vert_shader: vk::ShaderModule,
    vert_entry: *const c_char,
    vert_sizes: Vec<vk::VertexInputBindingDescription>,
    vert_input: Vec<vk::VertexInputAttributeDescription>,
    frag_shader: vk::ShaderModule,
    frag_entry: *const c_char,
    geom_shader: vk::ShaderModule,
    geom_entry: *const c_char,
    topology: pipeline::Topology,
    extent: vk::Extent2D,
    render_pass: vk::RenderPass,
    subpass_index: u32,
    cull_mode: pipeline::CullMode,
    src_color_blend_factor: vk::BlendFactor,
    dst_color_blend_factor: vk::BlendFactor,
    src_alpha_blend_factor: vk::BlendFactor,
    dst_alpha_blend_factor: vk::BlendFactor,
    line_width: f32,
    enable_depth_test: bool,
    enable_primitive_restart: bool,
    enable_dynamic_scissor: bool,
    enable_blend: bool,
    enable_dynamic_viewport: bool,
    enable_dynamic_line_width: bool
}

impl GraphicsPipelineBuilder {
    pub fn new() -> GraphicsPipelineBuilder {
        GraphicsPipelineBuilder {
            vert_shader: vk::ShaderModule::null(),
            vert_entry: std::ptr::null(),
            vert_sizes: Vec::new(),
            vert_input: Vec::new(),
            frag_shader: vk::ShaderModule::null(),
            frag_entry: std::ptr::null(),
            geom_shader: vk::ShaderModule::null(),
            geom_entry: std::ptr::null(),
            topology: pipeline::Topology::TRIANGLE_LIST,
            extent: memory::Extent2D::default(),
            render_pass: vk::RenderPass::null(),
            subpass_index: 0,
            cull_mode: pipeline::CullMode::BACK,
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            line_width: 1.0,
            enable_depth_test: false,
            enable_primitive_restart: false,
            enable_dynamic_scissor: false,
            enable_blend: false,
            enable_dynamic_viewport: false,
            enable_dynamic_line_width: false
        }
    }

    /// Must be called
    ///
    /// `shader` must outlive builder
    ///
    /// `vertex_size`
    ///
    /// For example you may pass just vertex coordinates in 3D space as `[f32; 3]`
    /// (vertex_size will be `size_of::<[f32; 3]>()`)
    /// or with 4-th coordinate as `[f32; 4]` (vertex_size will be `size_of::<[f32; 4]>()`)
    pub fn vertex_shader(&mut self, shader: &shader::Shader) -> &mut Self {
        self.vert_shader = shader.module();
        self.vert_entry = shader.entry().as_ptr();

        self
    }

    /// Optional
    ///
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
    /// use libvktypes::pipeline::GraphicsPipelineBuilder;
    ///
    /// struct Vertex {
    ///     pos: [f32; 4],
    ///     color: [f32; 4],
    /// }
    ///
    /// let mut builder = GraphicsPipelineBuilder::new();
    ///
    /// // Vertex
    /// builder.vertex_binding_input(0, std::mem::size_of::<[f32; 6]>() as u32);
    ///
    /// // Position
    /// builder.vertex_input(0, 0, ImageFormat::R32G32B32A32_SFLOAT, 0);
    ///
    /// // Color
    /// builder.vertex_input(1, 0, ImageFormat::R32G32B32A32_SFLOAT, std::mem::size_of::<[f32; 4]>() as u32);
    /// ```
    ///
    /// ## stride
    ///
    /// For example you may pass just vertex coordinates in 3D space as `[f32; 3]`
    /// (stride will be `size_of::<[f32; 3]>()`)
    /// or with 4-th coordinate as `[f32; 4]` (stride will be `size_of::<[f32; 4]>()`)
    pub fn vertex_input(&mut self,
        location: u32,
        binding: u32,
        format: memory::ImageFormat,
        offset: u32
    ) -> &mut Self {
        self.vert_input.push(
            vk::VertexInputAttributeDescription {
                location,
                binding,
                format,
                offset
            });

        self
    }

    /// Must be called
    ///
    /// Specify vertex size
    ///
    /// See [more](Self::vertex_input) on how to use
    pub fn vertex_binding_input(&mut self, binding: u32, stride: u32) -> &mut Self {
        self.vert_sizes.push(
            vk::VertexInputBindingDescription {
                binding,
                stride,
                input_rate: vk::VertexInputRate::VERTEX,
            });

        self
    }

    /// Must be called
    ///
    /// `shader` must outlive builder
    pub fn frag_shader(&mut self, shader: &shader::Shader) -> &mut Self {
        self.frag_shader = shader.module();
        self.frag_entry = shader.entry().as_ptr();

        self
    }

    /// Must be caled or [`extent2d`](Self::extent2d)
    pub fn extent(&mut self, width: u32, height: u32) -> &mut Self {
        self.extent = vk::Extent2D { width, height };

        self
    }

    /// Must be caled or [`extent`](Self::extent)
    pub fn extent2d(&mut self, extent2d: memory::Extent2D) -> &mut Self {
        self.extent = extent2d;

        self
    }

    /// Must be caled
    ///
    /// Render pass must outlive builder
    pub fn render_pass(&mut self, rp: &graphics::RenderPass) -> &mut Self {
        self.render_pass = rp.render_pass();

        self
    }

    /// Optional
    ///
    /// Subpass index inside [RenderPass](graphics::RenderPass)
    ///
    /// Default is `0`
    pub fn subpass(&mut self, idx: u32) -> &mut Self {
        self.subpass_index = idx;

        self
    }

    /// Optional
    ///
    /// `shader` must outlive builder
    pub fn geom_shader(&mut self, shader: &shader::Shader) -> &mut Self {
        self.geom_shader = shader.module();
        self.geom_entry = shader.entry().as_ptr();

        self
    }

    /// Optional
    ///
    /// A good explanation about topologies may be found
    /// [here](https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#drawing-primitive-topology-class)
    ///
    /// Default is `Topology::TRIANGLE_LIST`
    pub fn topology(&mut self, topology: pipeline::Topology) -> &mut Self {
        self.topology = topology;

        self
    }

    /// Optional
    ///
    /// Set flag to perform depth test
    ///
    /// However you have to allocate depth buffer and properly pass it to the render pass
    ///
    /// Default is `false`
    pub fn depth_test(&mut self, enable_depth_test: bool) -> &mut Self {
        self.enable_depth_test = enable_depth_test;

        self
    }

    /// Optional
    ///
    /// Affects [indexed drawing](crate::cmd::Buffer::draw_indexed)
    ///
    /// `primitive_restart` controls whether a special vertex index value is treated as restarting the assembly of primitives
    ///
    /// For example the special index value is
    /// [`INDEX_REASSEMBLY_UINT32`](memory::INDEX_REASSEMBLY_UINT32) for `IndexBufferType::UINT32`
    /// and so on
    ///
    /// Read more [here](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkPipelineInputAssemblyStateCreateInfo.html)
    ///
    /// Default is `false`
    pub fn primitive_restart(&mut self, enable_primitive_restart: bool) -> &mut Self {
        self.enable_primitive_restart = enable_primitive_restart;

        self
    }

    /// Optional
    ///
    /// Default is `CullMode::BACK`
    pub fn cull_mode(&mut self, cull_mode: pipeline::CullMode) -> &mut Self {
        self.cull_mode = cull_mode;

        self
    }

    /// Optional
    ///
    /// Default is `BlendFactor::ONE`
    pub fn src_color_blend_factor(&mut self, factor: pipeline::BlendFactor) -> &mut Self {
        self.src_color_blend_factor = factor;

        self
    }

    /// Optional
    ///
    /// Default is `BlendFactor::ZERO`
    pub fn dst_color_blend_factor(&mut self, factor: pipeline::BlendFactor) -> &mut Self {
        self.dst_color_blend_factor = factor;

        self
    }

    /// Optional
    ///
    /// Default is `BlendFactor::ONE`
    pub fn src_alpha_blend_factor(&mut self, factor: pipeline::BlendFactor) -> &mut Self {
        self.src_alpha_blend_factor = factor;

        self
    }

    /// Optional
    ///
    /// Default is `BlendFactor::ZERO`
    pub fn dst_alpha_blend_factor(&mut self, factor: pipeline::BlendFactor) -> &mut Self {
        self.dst_alpha_blend_factor = factor;

        self
    }

    /// Optional
    ///
    /// Default is `false`
    pub fn dynamic_scissor(&mut self, enable: bool) -> &mut Self {
        self.enable_dynamic_scissor = enable;

        self
    }

    /// Optional
    ///
    /// Default is `false`
    pub fn blend(&mut self, enable: bool) -> &mut Self {
        self.enable_blend = enable;

        self
    }

    /// Optional
    ///
    /// Default is `false`
    pub fn dynamic_viewport(&mut self, enable: bool) -> &mut Self {
        self.enable_dynamic_viewport = enable;

        self
    }

    /// Optional
    ///
    /// Default is `false`
    pub fn dynamic_line_width(&mut self, enable: bool) -> &mut Self {
        self.enable_dynamic_line_width = enable;

        self
    }

    /// Optional
    ///
    /// Default is `1.0`
    pub fn line_width(&mut self, width: f32) -> &mut Self {
        self.line_width = width;

        self
    }

    /// Try to create pipeline
    pub fn build(&self,
        device: &dev::Device,
        layout: &pipeline::PipelineLayout
    ) -> Result<GraphicsPipeline, pipeline::PipelineError> {
        use std::marker::PhantomData;

        // Shaders
        let mut shader_stage_create_infos = vec![
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                stage: vk::ShaderStageFlags::VERTEX,
                module: self.vert_shader,
                p_name: self.vert_entry,
                p_specialization_info: std::ptr::null(),
                _marker: PhantomData,
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                module: self.frag_shader,
                p_name: self.frag_entry,
                p_specialization_info: std::ptr::null(),
                _marker: PhantomData,
            },
        ];

        if self.geom_shader != vk::ShaderModule::null() {
            shader_stage_create_infos.push(
                vk::PipelineShaderStageCreateInfo {
                    s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                    p_next: std::ptr::null(),
                    flags: vk::PipelineShaderStageCreateFlags::empty(),
                    stage: vk::ShaderStageFlags::GEOMETRY,
                    module: self.geom_shader,
                    p_name: self.geom_entry,
                    p_specialization_info: std::ptr::null(),
                    _marker: PhantomData,
                }
            );
        }

        let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineVertexInputStateCreateFlags::empty(),
            vertex_binding_description_count: self.vert_sizes.len() as u32,
            p_vertex_binding_descriptions: data_ptr!(self.vert_sizes),
            vertex_attribute_description_count: self.vert_input.len() as u32,
            p_vertex_attribute_descriptions: data_ptr!(self.vert_input),
            _marker: PhantomData,
        };

        let input_assembly_state_create_info = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
            topology: self.topology,
            primitive_restart_enable: self.enable_primitive_restart as ash::vk::Bool32,
            _marker: PhantomData,
        };

        let viewports = [vk::Viewport {
            x: 0_f32,
            y: 0_f32,
            width: self.extent.width as f32,
            height: self.extent.height as f32,
            min_depth: 0_f32,
            max_depth: 1_f32,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.extent,
        }];

        /*
            Now we must specify the form of output data
            Viewport specifies to what part of the image (or texture, or window) we want do draw
        */
        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineViewportStateCreateFlags::empty(),
            viewport_count: viewports.len() as u32,
            p_viewports: data_ptr!(viewports),
            scissor_count: scissors.len() as u32,
            p_scissors: data_ptr!(scissors),
            _marker: PhantomData,
        };

        /*
            The next part of the graphics pipeline creation applies to the rasterization state
            We must specify how polygons are going to be rasterized (changed into fragments)
        */
        let rasterization_state_create_info = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable: ash::vk::FALSE,
            rasterizer_discard_enable: ash::vk::FALSE,
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: self.cull_mode,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            depth_bias_enable: ash::vk::FALSE,
            depth_bias_constant_factor: 0.0,
            depth_bias_clamp: 0.0,
            depth_bias_slope_factor: 0.0,
            line_width: self.line_width,
            _marker: PhantomData,
        };

        /*
            In Vulkan, when we are creating a graphics pipeline, we must also specify the state relevant to multisampling
        */
        let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineMultisampleStateCreateFlags::empty(),
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            sample_shading_enable: ash::vk::FALSE,
            min_sample_shading: 1.0,
            p_sample_mask: std::ptr::null(),
            alpha_to_coverage_enable: ash::vk::FALSE,
            alpha_to_one_enable: ash::vk::FALSE,
            _marker: PhantomData,
        };

        let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState {
            blend_enable: if self.enable_blend { ash::vk::TRUE } else { ash::vk::FALSE },
            src_color_blend_factor: self.src_color_blend_factor,
            dst_color_blend_factor: self.dst_color_blend_factor,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: self.src_alpha_blend_factor,
            dst_alpha_blend_factor: self.dst_alpha_blend_factor,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        };

        let color_blend_state_create_info = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable: ash::vk::FALSE,
            logic_op: vk::LogicOp::COPY,
            attachment_count: 1,
            p_attachments: &color_blend_attachment_state,
            blend_constants: [0.0; 4],
            _marker: PhantomData,
        };

        let depth_cfg = vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
            depth_test_enable: if self.enable_depth_test { ash::vk::TRUE } else { ash::vk::FALSE },
            depth_write_enable: ash::vk::TRUE,
            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
            depth_bounds_test_enable: 0,
            stencil_test_enable: 0,
            front: vk::StencilOpState::default(),
            back: vk::StencilOpState::default(),
            min_depth_bounds: f32::default(),
            max_depth_bounds: f32::default(),
            _marker: PhantomData,
        };

        let mut dynamic_states: Vec<vk::DynamicState> = Vec::new();

        if self.enable_dynamic_scissor {
            dynamic_states.push(vk::DynamicState::SCISSOR);
        }

        if self.enable_dynamic_viewport {
            dynamic_states.push(vk::DynamicState::VIEWPORT);
        }

        if self.enable_dynamic_line_width {
            dynamic_states.push(vk::DynamicState::LINE_WIDTH);
        }

        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineDynamicStateCreateFlags::empty(),
            dynamic_state_count: dynamic_states.len() as u32,
            p_dynamic_states: data_ptr!(&dynamic_states),
            _marker: PhantomData,
        };

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineCreateFlags::empty(),
            stage_count: shader_stage_create_infos.len() as u32,
            p_stages: shader_stage_create_infos.as_ptr(),
            p_vertex_input_state: &vertex_input_state_create_info,
            p_input_assembly_state: &input_assembly_state_create_info,
            p_tessellation_state: std::ptr::null(),
            p_viewport_state: &viewport_state_create_info,
            p_rasterization_state: &rasterization_state_create_info,
            p_multisample_state: &multisample_state_create_info,
            p_depth_stencil_state: &depth_cfg,
            p_color_blend_state: &color_blend_state_create_info,
            p_dynamic_state: &dynamic_state_info,
            layout: layout.layout(),
            render_pass: self.render_pass,
            subpass: self.subpass_index,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: -1,
            _marker: PhantomData,
        };

        let pipeline = unsafe { on_error_ret!(
            device
            .device()
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[pipeline_create_info],
                device.allocator()
            ), pipeline::PipelineError::Pipeline
        )};


        Ok(GraphicsPipeline {
            i_core: device.core().clone(),
            i_pipeline: pipeline[0]
        })
    }
}

/// Graphics pipeline
#[derive(Debug)]
pub struct GraphicsPipeline {
    i_core: Arc<dev::Core>,
    i_pipeline: vk::Pipeline
}

impl GraphicsPipeline {
    pub(crate) fn pipeline(&self) -> vk::Pipeline {
        self.i_pipeline
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        let device = self.i_core.device();
        let alloc  = self.i_core.allocator();

        unsafe {
            device.destroy_pipeline(self.i_pipeline, alloc);
        }
    }
}
