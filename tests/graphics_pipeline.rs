mod test_context;

#[cfg(test)]
mod graphics_pipeline {
    use libvktypes::{graphics, memory, hw};

    use super::test_context;

    #[test]
    fn create_pipeline() {
        let dev = test_context::get_graphics_device();

        let capabilities = test_context::get_surface_capabilities();

        let pipe_type = graphics::PipelineCfg {
            vertex_shader: test_context::get_vert_shader(),
            vertex_size: std::mem::size_of::<[f32; 2]>() as u32,
            vert_input: &[],
            frag_shader: test_context::get_frag_shader(),
            topology: graphics::Topology::TRIANGLE_STRIP,
            extent: capabilities.extent2d(),
            push_constant_size: 0,
            render_pass: test_context::get_render_pass(),
            subpass_index: 0,
            enable_depth_test: false,
            enable_primitive_restart: false,
            cull_mode: graphics::CullMode::BACK,
            descriptor: &graphics::PipelineDescriptor::empty(dev)
        };

        assert!(graphics::Pipeline::new(dev, &pipe_type).is_ok());
    }

    #[test]
    fn with_resources() {
        let capabilities = test_context::get_surface_capabilities();

        let device = test_context::get_graphics_device();

        let descs = graphics::PipelineDescriptor::allocate(&device, &[&[
            graphics::BindingCfg {
                resource_type: graphics::ResourceType::UNIFORM_BUFFER,
                stage: graphics::ShaderStage::VERTEX | graphics::ShaderStage::FRAGMENT,
                count: 1,
            }
        ]]).expect("Failed to allocate resources");

        let pipe_type = graphics::PipelineCfg {
            vertex_shader: test_context::get_vert_shader(),
            vertex_size: std::mem::size_of::<[f32; 2]>() as u32,
            vert_input: &[],
            frag_shader: test_context::get_frag_shader(),
            topology: graphics::Topology::TRIANGLE_STRIP,
            extent: capabilities.extent2d(),
            push_constant_size: 0,
            render_pass: test_context::get_render_pass(),
            subpass_index: 0,
            enable_depth_test: false,
            enable_primitive_restart: false,
            cull_mode: graphics::CullMode::BACK,
            descriptor: &descs
        };

        assert!(graphics::Pipeline::new(device, &pipe_type).is_ok());
    }

    #[test]
    fn write_resource() {
        let device = test_context::get_graphics_device();

        let queue = test_context::get_graphics_queue();

        let mem_cfg = memory::MemoryCfg {
            properties: hw::MemoryProperty::HOST_VISIBLE,
            filter: &hw::any,
            buffers: &[
                &memory::BufferCfg {
                    size: 16,
                    usage: memory::UNIFORM,
                    queue_families: &[queue.index()],
                    simultaneous_access: false,
                    count: 1
                }
            ]
        };

        let uniform_data = memory::Memory::allocate(&device, &mem_cfg).expect("Failed to allocate memory");

        let descs = graphics::PipelineDescriptor::allocate(&device, &[&[
            graphics::BindingCfg {
                resource_type: graphics::ResourceType::UNIFORM_BUFFER,
                stage: graphics::ShaderStage::VERTEX | graphics::ShaderStage::FRAGMENT,
                count: 1,
            }
        ]]).expect("Failed to allocate resources");

        descs.update(&[&[&[&uniform_data.view(0)]]])
    }

    #[test]
    fn default_sampler() {
        let device = test_context::get_graphics_device();

        let cfg = graphics::SamplerCfg::default();

        assert!(graphics::Sampler::new(device, &cfg).is_ok());
    }
}