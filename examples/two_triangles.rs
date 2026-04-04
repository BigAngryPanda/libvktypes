use libvktypes::*;

const VERT_SHADER: &str = "
#version 460

layout (location=0) in vec4 inPos;
layout (location = 0) out vec4 outPos;

void main() {
    outPos = inPos;
}
";

const GEOM_SHADER: &str = "
#version 460

layout(triangles) in;
layout(triangle_strip, max_vertices=6) out;

layout (location = 0) in vec4 inPos[];

layout (location = 0) out vec4 colorData;

void main() {
    int i;
    for(i=0; i<3; i++)
    {
        gl_Position = inPos[i];
        colorData = i*vec4(0.3, 0.3, 0.3, 0.0);
        EmitVertex();
    }
    EndPrimitive();

    for(i=0; i<3; i++)
    {
        gl_Position = vec4(1.0, 1.0, 0.0, 0.0) + inPos[i];
        colorData = i*vec4(0, 0.3, 0, 0.0);
        EmitVertex();
    }
    EndPrimitive();
}
";

const FRAG_SHADER: &str = "
#version 460

layout (location=0) in vec4 inColor;

layout (location=0) out vec4 outColor;

void main(){
    outColor = inColor;
}
";

fn main() {
    let data = [
        0.0f32, -1f32, 0.0f32, 1.0f32,
        -1f32, -1f32, 0.0f32, 1.0f32,
        -1f32, 0f32, 0.0f32, 1.0f32,
    ];

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

    let swapchain = swapchain::Swapchain::new(&lib, &device, &surface, &swp_type, None).expect("Failed to create swapchain");

    let vert_shader = shader::ShaderBuilder::new()
        .path("VERT_SHADER")
        .glsl_src(VERT_SHADER)
        .shader_type(shader::Kind::Vertex)
        .from_glsl(&device)
        .expect("Failed to create vertex shader module");

    let frag_shader = shader::ShaderBuilder::new()
        .path("FRAG_SHADER")
        .glsl_src(FRAG_SHADER)
        .shader_type(shader::Kind::Fragment)
        .from_glsl(&device)
        .expect("Failed to create vertex shader module");

    let geom_shader = shader::ShaderBuilder::new()
        .path("GEOM_SHADER")
        .glsl_src(GEOM_SHADER)
        .shader_type(shader::Kind::Geometry)
        .from_glsl(&device)
        .expect("Failed to create vertex shader module");

    let buffers = [
        memory::LayoutElementCfg::Buffer {
            size: (std::mem::size_of::<f32>()*data.len()) as u64,
            usage: memory::VERTEX,
            queue_families: &[queue.index()],
            simultaneous_access: false,
            count: 1
        }
    ];

    let vertex_data = memory::Memory::allocate_host_memory(&device, &mut buffers.iter()).expect("Failed to allocate memory");

    let mut set_vrtx_buffer = |bytes: &mut [f32]| {
        bytes.clone_from_slice(&data);
    };

    vertex_data.access(&mut set_vrtx_buffer, 0).expect("Failed to fill the buffer");

    let render_pass = graphics::RenderPass::single_subpass(&device, surf_format, memory::SampleCountFlags::TYPE_1)
        .expect("Failed to create render pass");

    let layout = pipeline::PipelineLayoutBuilder::new()
        .build(&device)
        .expect("Failed to crate pipeline layout");

    let pipeline = pipeline::GraphicsPipelineBuilder::new()
        .vertex_shader(&vert_shader)
        .vertex_binding_input(0, std::mem::size_of::<[f32; 4]>() as u32)
        .vertex_input(0, 0, memory::ImageFormat::R32G32B32A32_SFLOAT, 0)
        .geom_shader(&geom_shader)
        .frag_shader(&frag_shader)
        .render_pass(&render_pass)
        .extent2d(capabilities.extent2d())
        .build(&device, &layout)
        .expect("failed to create pipeline");

    let img_sem = sync::Semaphore::new(&device).expect("Failed to create semaphore");
    let render_sem = sync::Semaphore::new(&device).expect("Failed to create semaphore");

    let cmd_pool = cmd::Pool::new(&device, queue.index()).expect("Failed to allocate command pool");

    let cmd_buffer = cmd_pool.allocate().expect("Failed to allocate command pool");

    let images = swapchain.images().expect("Failed to get images");

    let img_index = swapchain.next_image(u64::MAX, Some(&img_sem), None).expect("Failed to get image index");

    let image_views = [
        memory::view::RefImageView::new(&images[img_index as usize], 0)
    ];

    let mut framebuffer_cfg = memory::FramebufferCfg {
        images: &mut image_views.iter(),
        extent: capabilities.extent2d(),
        render_pass: &render_pass,
    };

    let framebuffer = memory::Framebuffer::new(&device, &mut framebuffer_cfg).expect("Failed to create framebuffer");

    cmd_buffer.begin_render_pass(&render_pass, &framebuffer);

    cmd_buffer.bind_graphics_pipeline(&pipeline);

    cmd_buffer.bind_vertex_buffers(&[memory::RefView::new(&vertex_data, 0)]);

    cmd_buffer.draw(3, 1, 0, 0);

    cmd_buffer.end_render_pass();

    let exec_buffer = cmd_buffer.commit().expect("Failed to commit buffer");

    let cmd_queue = queue::Queue::new(&device, queue.index(), 0);

    let fence = sync::Fence::new(&device, false).expect("Failed to create fence");

    let exec_info = queue::ExecInfo {
        buffer: &exec_buffer,
        wait_stage: cmd::PipelineStage::COLOR_ATTACHMENT_OUTPUT,
        timeout: u64::MAX,
        wait: &[&img_sem],
        signal: &[&render_sem],
        fence: &fence
    };

    cmd_queue.exec(&exec_info).expect("Failed to execute queue");

    let present_info = queue::PresentInfo {
        swapchain: &swapchain,
        image_index: img_index,
        wait: &[&render_sem]
    };

    cmd_queue.present(&present_info).expect("Failed to present frame");

    device.wait_for_fence(&fence, u64::MAX).expect("Failed to wait or reset Fence");

    event_loop.run(move |event, control_flow| {
        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => {
                device.wait_idle().expect("Failed to wait idle");
                control_flow.exit();
            },
            _ => ()
        }

    }).expect("Failed to run example");
}