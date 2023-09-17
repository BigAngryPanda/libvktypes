use libvktypes::*;

fn main() {
    let event_loop = window::eventloop();

    let wnd = window::create_window(&event_loop).expect("Failed to create window");

    let lib_type = libvk::InstanceType {
        debug_layer: Some(layers::DebugLayer::default()),
        extensions: &[extensions::DEBUG_EXT_NAME,
            extensions::SURFACE_EXT_NAME,
            extensions::XLIB_SURFACE_EXT_NAME],
        ..libvk::InstanceType::default()
    };

    let lib = libvk::Instance::new(&lib_type).expect("Failed to load library");

    let surface = surface::Surface::new(&lib, &wnd).expect("Failed to create surface");

    let hw_list = hw::Description::poll(&lib, Some(&surface)).expect("Failed to list hardware");

    let (hw_dev, queue, _) = hw_list
        .find_first(
            hw::HWDevice::is_discrete_gpu,
            |q| q.is_graphics() && q.is_surface_supported(),
            |_| true
        )
        .expect("Failed to find suitable hardware device");

    let dev_type = dev::DeviceCfg {
        lib: &lib,
        hw: hw_dev,
        extensions: &[extensions::SWAPCHAIN_EXT_NAME],
        allocator: None,
    };

    let device = dev::Device::new(&dev_type).expect("Failed to create device");

    let capabilities = surface::Capabilities::get(&hw_dev, &surface).expect("Failed to get capabilities");

    assert!(capabilities.is_mode_supported(swapchain::PresentMode::FIFO));
    assert!(capabilities.is_flags_supported(memory::UsageFlags::COLOR_ATTACHMENT));

    let surf_format = capabilities.formats().next().expect("No available formats").format;

    let swp_type = swapchain::SwapchainCfg {
        num_of_images: capabilities.min_img_count(),
        format: surf_format,
        color: memory::ColorSpace::SRGB_NONLINEAR,
        present_mode: swapchain::PresentMode::FIFO,
        flags: memory::UsageFlags::COLOR_ATTACHMENT,
        extent: capabilities.extent2d(),
        transform: capabilities.pre_transformation(),
        alpha: capabilities.first_alpha_composition().expect("No alpha composition")
    };

    let swapchain = swapchain::Swapchain::new(&lib, &device, &surface, &swp_type).expect("Failed to create swapchain");

    let vert_shader_type = shader::ShaderCfg {
        path: "examples/compiled_shaders/vertex_input.spv",
        entry: "main",
    };

    let vert_shader = shader::Shader::from_file(&device, &vert_shader_type).expect("Failed to create vertex shader module");

    let frag_shader_type = shader::ShaderCfg {
        path: "examples/compiled_shaders/color_from_vertex.spv",
        entry: "main",
    };

    let frag_shader = shader::Shader::from_file(&device, &frag_shader_type).expect("Failed to create fragment shader module");

    let mem_cfg = memory::MemoryCfg {
        properties: hw::MemoryProperty::HOST_VISIBLE,
        filter: &hw::any,
        buffers: &[
            &memory::BufferCfg {
                size: 16*4,
                usage: memory::VERTEX,
                queue_families: &[queue.index()],
                simultaneous_access: false,
                count: 1
            }
        ]
    };

    let vertex_data = memory::Memory::allocate(&device, &mem_cfg).expect("Failed to allocate memory");

    let mut set_vrtx_buffer = |bytes: &mut [f32]| {
        bytes.clone_from_slice(&[0.5f32, 0.5f32, 0.0f32, 1.0f32,
                     0.5f32, -0.5f32, 0.0f32, 1.0f32,
                     -0.5f32, 0.5f32, 0.0f32, 1.0f32,
                     -0.5f32, -0.5f32, 0.0f32, 1.0f32,]);
    };

    vertex_data.access(&mut set_vrtx_buffer, 0).expect("Failed to fill the buffer");

    let render_pass = graphics::RenderPass::single_subpass(&device, surf_format)
        .expect("Failed to create render pass");

    let pipe_type = graphics::PipelineCfg {
        vertex_shader: &vert_shader,
        vertex_size: std::mem::size_of::<[f32; 4]>() as u32,
        vert_input: &[graphics::VertexInputCfg {
            location: 0,
            binding: 0,
            format: memory::ImageFormat::R32G32B32A32_SFLOAT,
            offset: 0,
        }],
        frag_shader: &frag_shader,
        topology: graphics::Topology::TRIANGLE_STRIP,
        extent: capabilities.extent2d(),
        push_constant_size: 0,
        render_pass: &render_pass,
        subpass_index: 0,
        enable_depth_test: false,
        enable_primitive_restart: false,
        cull_mode: graphics::CullMode::BACK,
        sets: &[]
    };

    let pipeline = graphics::Pipeline::new(&device, &pipe_type).expect("Failed to create pipeline");

    let img_sem = sync::Semaphore::new(&device).expect("Failed to create semaphore");
    let render_sem = sync::Semaphore::new(&device).expect("Failed to create semaphore");

    let cmd_pool_type = cmd::PoolCfg {
        queue_index: queue.index(),
    };

    let cmd_pool = cmd::Pool::new(&device, &cmd_pool_type).expect("Failed to allocate command pool");

    let cmd_buffer = cmd_pool.allocate().expect("Failed to allocate command pool");

    let images = swapchain.images().expect("Failed to get images");

    let img_index = swapchain.next_image(u64::MAX, Some(&img_sem), None).expect("Failed to get image index");

    let frames_cfg = memory::FramebufferCfg {
        render_pass: &render_pass,
        images: &[images[img_index as usize].view(0)],
        extent: capabilities.extent2d(),
    };

    let frame = memory::Framebuffer::new(&device, &frames_cfg).expect("Failed to create framebuffers");

    cmd_buffer.begin_render_pass(&render_pass, &frame);

    cmd_buffer.bind_graphics_pipeline(&pipeline);

    cmd_buffer.bind_vertex_buffers(&[vertex_data.vertex_view(0, 0)]);

    cmd_buffer.draw(4, 1, 0, 0);

    cmd_buffer.end_render_pass();

    let exec_buffer = cmd_buffer.commit().expect("Failed to commit buffer");

    let queue_cfg = queue::QueueCfg {
        family_index: queue.index(),
        queue_index: 0
    };

    let cmd_queue = queue::Queue::new(&device, &queue_cfg);

    let exec_info = queue::ExecInfo {
        buffer: &exec_buffer,
        wait_stage: cmd::PipelineStage::COLOR_ATTACHMENT_OUTPUT,
        timeout: u64::MAX,
        wait: &[&img_sem],
        signal: &[&render_sem],
    };

    cmd_queue.exec(&exec_info).expect("Failed to execute queue");

    let present_info = queue::PresentInfo {
        swapchain: &swapchain,
        image_index: img_index,
        wait: &[&render_sem]
    };

    cmd_queue.present(&present_info).expect("Failed to present frame");

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => {
                control_flow.set_exit();
            },
            _ => ()
        }

    });
}