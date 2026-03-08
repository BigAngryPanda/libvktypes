mod test_context;

#[cfg(test)]
mod graphics_pipeline {
    use libvktypes::{
        graphics,
        memory,
        pipeline
    };

    use super::test_context;

    #[test]
    fn create_pipeline() {
        let device = test_context::get_graphics_device();

        let capabilities = test_context::get_surface_capabilities();

        let layout = pipeline::PipelineLayoutBuilder::new()
            .build(&device)
            .expect("Failed to crate pipeline layout");

        let pipe = pipeline::GraphicsPipelineBuilder::new()
            .vertex_shader(test_context::get_vert_shader())
            .frag_shader(test_context::get_frag_shader())
            .render_pass(test_context::get_render_pass())
            .extent(capabilities.extent2d().width, capabilities.extent2d().height)
            .build(&device, &layout);

        assert!(pipe.is_ok());
    }

    #[test]
    fn with_resources() {
        let capabilities = test_context::get_surface_capabilities();

        let device = test_context::get_graphics_device();

        let layout = pipeline::PipelineLayoutBuilder::with_sets(1)
            .binding(0, 0, graphics::ShaderStage::VERTEX | graphics::ShaderStage::FRAGMENT,
                pipeline::DescriptorType::UNIFORM_BUFFER, 1)
            .build(&device)
            .expect("Failed to crate pipeline layout");

        let pipe = pipeline::GraphicsPipelineBuilder::new()
            .vertex_shader(test_context::get_vert_shader())
            .frag_shader(test_context::get_frag_shader())
            .render_pass(test_context::get_render_pass())
            .extent(capabilities.extent2d().width, capabilities.extent2d().height)
            .build(&device, &layout);

        assert!(pipe.is_ok());
    }

    #[test]
    fn write_resource() {
        let device = test_context::get_graphics_device();

        let queue = test_context::get_graphics_queue();

        let buffers = [
            memory::LayoutElementCfg::Buffer {
                size: 16,
                usage: memory::UNIFORM,
                queue_families: &[queue.index()],
                simultaneous_access: false,
                count: 1
            }
        ];

        let uniform_data = memory::Memory::allocate_host_memory(
            &device, &mut buffers.iter()).expect("Failed to allocate memory");

        let layout = pipeline::PipelineLayoutBuilder::with_sets(1)
            .binding(0, 0, graphics::ShaderStage::VERTEX | graphics::ShaderStage::FRAGMENT,
                pipeline::DescriptorType::UNIFORM_BUFFER, 1)
            .build(&device)
            .expect("Failed to crate pipeline layout");

        let bindings = pipeline::PipelineBindings::new(&device, &layout).expect("Failed to create bindings");

        let mut write_info = pipeline::WriteInfo::new();
        write_info
            .buffer(0, 0, pipeline::DescriptorType::STORAGE_BUFFER)
            .element(memory::RefView::new(&uniform_data, 0));

        bindings.write(&write_info);
    }

    #[test]
    fn default_sampler() {
        let device = test_context::get_graphics_device();

        let cfg = graphics::SamplerCfg::default();

        assert!(graphics::Sampler::new(device, &cfg).is_ok());
    }
}