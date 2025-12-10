use libvktypes::{
    window,
    libvk,
    layers,
    extensions,
    surface,
    hw,
    dev,
    swapchain,
    memory,
    shader,
    graphics,
    sync,
    cmd,
    queue
};

const VERT_SHADER: &str = "
#version 460

layout (location=0) in vec4 position;

void main() {
    gl_Position = position;
}
";

const FRAG_SHADER: &str = "
#version 460

layout (location=0) out vec4 color;

layout(set=0, binding=0) uniform Data {
    vec4 colour;
} data[2];

void main(){
    color = data[0].colour + data[1].colour;
}
";

fn main() {
    let event_loop = window::eventloop().expect("Failed to create eventloop");

    let wnd = window::create_window(&event_loop).expect("Failed to create window");

    let mut extensions = extensions::required_extensions(&wnd);
    extensions.push(extensions::DEBUG_EXT_NAME);
    extensions.push(extensions::SURFACE_EXT_NAME);

    let lib_type = libvk::InstanceType {
        debug_layer: Some(layers::DebugLayer::default()),
        extensions: &extensions,
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
        path: "VERT_DATA",
        entry: "main",
    };

    let vert_shader =
        shader::Shader::from_glsl(&device, &vert_shader_type, VERT_SHADER, shader::Kind::Vertex)
        .expect("Failed to create vertex shader module");

    let frag_shader_type = shader::ShaderCfg {
        path: "FRAG_DATA",
        entry: "main",
    };

    let frag_shader =
        shader::Shader::from_glsl(&device, &frag_shader_type, FRAG_SHADER, shader::Kind::Fragment)
        .expect("Failed to create fragment shader module");

    let buffers = [
        memory::LayoutElementCfg::Buffer(memory::BufferCfg {
            size: 4*std::mem::size_of::<[f32; 4]>() as u64,
            usage: memory::VERTEX,
            queue_families: &[queue.index()],
            simultaneous_access: false,
            count: 1
        }),
        memory::LayoutElementCfg::Buffer(memory::BufferCfg {
            size: std::mem::size_of::<[f32; 4]>() as u64,
            usage: memory::UNIFORM,
            queue_families: &[queue.index()],
            simultaneous_access: false,
            count: 2
        })
    ];

    let data = memory::Memory::allocate_host_memory(&device, &mut buffers.iter()).expect("Failed to allocate memory");

    let vertices = memory::RefView::new(&data, 0);
    let first_color  = memory::RefView::new(&data, 1);
    let second_color  = memory::RefView::new(&data, 1);

    let mut set_vrtx_buffer = |bytes: &mut [f32]| {
        bytes.clone_from_slice(&[
            0.5f32, 0.5f32, 0.0f32, 1.0f32,
            0.5f32, -0.5f32, 0.0f32, 1.0f32,
            -0.5f32, 0.5f32, 0.0f32, 1.0f32,
            -0.5f32, -0.5f32, 0.0f32, 1.0f32]);
    };

    data.access(&mut set_vrtx_buffer, 0).expect("Failed to fill the buffer");

    data.access(&mut |bytes: &mut [f32]| {
        bytes.clone_from_slice(
            &[
                0.4, 0.4, 0.4, 1.0
            ]
        );
    }, 1)
    .expect("Failed to fill the ubo");

    data.access(&mut |bytes: &mut [f32]| {
        bytes.clone_from_slice(
            &[
                0.12, 0.12, 0.12, 1.0
            ]
        );
    }, 2)
    .expect("Failed to fill the ubo");

    let render_pass = graphics::RenderPass::single_subpass(&device, surf_format)
        .expect("Failed to create render pass");

    let descs = graphics::PipelineDescriptor::allocate(&device, &[&[
        graphics::BindingCfg {
            resource_type: graphics::DescriptorType::UNIFORM_BUFFER,
            stage: graphics::ShaderStage::FRAGMENT,
            count: 2,
        }
    ]]).expect("Failed to allocate resources");

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
        geom_shader: None,
        topology: graphics::Topology::TRIANGLE_STRIP,
        extent: capabilities.extent2d(),
        push_constant_size: 0,
        render_pass: &render_pass,
        subpass_index: 0,
        enable_depth_test: false,
        enable_primitive_restart: false,
        cull_mode: graphics::CullMode::BACK,
        descriptor: &descs
    };

    let pipeline = graphics::Pipeline::new(&device, &pipe_type).expect("Failed to create pipeline");

    descs.update(&[graphics::UpdateInfo {
        set: 0,
        binding: 0,
        starting_array_element: 0,
        resources: graphics::ShaderBinding::Buffers::<_, memory::RefImageView>(&[
            graphics::BufferBinding::new(first_color),
            graphics::BufferBinding::new(second_color)
        ]),
    }]);

    let img_sem = sync::Semaphore::new(&device).expect("Failed to create semaphore");
    let render_sem = sync::Semaphore::new(&device).expect("Failed to create semaphore");

    let cmd_pool_type = cmd::PoolCfg {
        queue_index: queue.index(),
    };

    let cmd_pool = cmd::Pool::new(&device, &cmd_pool_type).expect("Failed to allocate command pool");

    let cmd_buffer = cmd_pool.allocate().expect("Failed to allocate command pool");

    let images = swapchain.images().expect("Failed to get images");

    let img_index = swapchain.next_image(u64::MAX, Some(&img_sem), None).expect("Failed to get image index");

    let image_views = [
        memory::view::RefImageView::new(&images[img_index as usize], 0)
    ];

    let mut frames_cfg = memory::FramebufferCfg {
        render_pass: &render_pass,
        images: &mut image_views.iter(),
        extent: capabilities.extent2d(),
    };

    let frame = memory::Framebuffer::new(&device, &mut frames_cfg).expect("Failed to create framebuffers");

    cmd_buffer.begin_render_pass(&render_pass, &frame);

    cmd_buffer.bind_graphics_pipeline(&pipeline);

    cmd_buffer.bind_vertex_buffers(&[graphics::VertexView::new(vertices)]);

    cmd_buffer.bind_resources(&pipeline, &descs, &[]);

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

    event_loop.run(move |event, control_flow| {
        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => {
                control_flow.exit();
            },
            _ => ()
        }

    }).expect("Failed to run example");
}