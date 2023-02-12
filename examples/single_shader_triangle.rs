use libvktypes::*;

use std::ffi::CString;

fn main() {
    let lib_type = libvk::InstanceType {
        debug_layer: Some(layers::DebugLayer::default()),
        extensions: &[extensions::DEBUG_EXT_NAME,
            extensions::SURFACE_EXT_NAME,
            extensions::XLIB_SURFACE_EXT_NAME],
        ..libvk::InstanceType::default()
    };

    let lib = libvk::Instance::new(&lib_type).expect("Failed to load library");

    let wnd = window::Window::new().expect("Failed to create window");

    let surface_cfg = surface::SurfaceType {
        lib: &lib,
        window: &wnd,
    };

    let surface = surface::Surface::new(&surface_cfg).expect("Failed to create surface");

    let hw_list = hw::Description::poll(&lib, Some(&surface)).expect("Failed to list hardware");

    let (hw_dev, queue, _) = hw_list
        .find_first(
            hw::HWDevice::is_dedicated_gpu,
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

    let cap_type = surface::CapabilitiesType {
        hw: hw_dev,
        surface: &surface
    };

    let capabilities = surface::Capabilities::get(&cap_type).expect("Failed to get capabilities");

    assert!(capabilities.is_mode_supported(surface::PresentMode::FIFO));
    assert!(capabilities.is_flags_supported(surface::UsageFlags::COLOR_ATTACHMENT));

    let surf_format = capabilities.formats().next().expect("No available formats").format;

    let swp_type = swapchain::SwapchainType {
        lib: &lib,
        dev: &device,
        surface: &surface,
        num_of_images: capabilities.min_img_count(),
        format: surf_format,
        color: surface::ColorSpace::SRGB_NONLINEAR,
        present_mode: surface::PresentMode::FIFO,
        flags: surface::UsageFlags::COLOR_ATTACHMENT,
        extent: capabilities.extent2d(),
        transform: capabilities.pre_transformation(),
        alpha: capabilities.first_alpha_composition().expect("No alpha composition")
    };

    let swapchain = swapchain::Swapchain::new(&swp_type).expect("Failed to create swapchain");

    let vert_shader_type = shader::ShaderType {
        device: &device,
        path: "examples/compiled_shaders/single_triangle.spv",
        entry: CString::new("main").expect("Failed to allocate string"),
    };

    let vert_shader = shader::Shader::from_file(&vert_shader_type).expect("Failed to create vertex shader module");

    let frag_shader_type = shader::ShaderType {
        device: &device,
        path: "examples/compiled_shaders/single_color.spv",
        entry: CString::new("main").expect("Failed to allocate string"),
    };

    let frag_shader = shader::Shader::from_file(&frag_shader_type).expect("Failed to create fragment shader module");

    let render_pass = graphics::RenderPass::single_subpass(&device, surf_format)
        .expect("Failed to create render pass");

    let pipe_type = graphics::PipelineType {
        vertex_shader: &vert_shader,
        vertex_size: std::mem::size_of::<[f32; 4]>() as u32,
        vert_input: &[],
        frag_shader: &frag_shader,
        topology: graphics::Topology::TRIANGLE_STRIP,
        extent: capabilities.extent2d(),
        push_constant_size: 0,
        render_pass: &render_pass,
        subpass_index: 0,
        enable_depth: false
    };

    let pipeline = graphics::Pipeline::new(&device, &pipe_type).expect("Failed to create pipeline");

    let sem_type = sync::SemaphoreType {
        device: &device,
    };

    let img_sem = sync::Semaphore::new(&sem_type).expect("Failed to create semaphore");
    let render_sem = sync::Semaphore::new(&sem_type).expect("Failed to create semaphore");

    let cmd_pool_type = cmd::PoolCfg {
        queue_index: queue.index(),
    };

    let cmd_pool = cmd::Pool::new(&device, &cmd_pool_type).expect("Failed to allocate command pool");

    let cmd_buffer = cmd_pool.allocate().expect("Failed to allocate command pool");

    let img_cfg = memory::ImageListType {
        device: &device,
        swapchain: &swapchain,
    };

    let images = memory::ImageList::from_swapchain(&img_cfg).expect("Failed to get images");

    let img_index = swapchain.next_image(u64::MAX, Some(&img_sem), None).expect("Failed to get image index");

    let frames_cfg = memory::FramebufferListType {
        device: &device,
        render_pass: &render_pass,
        images: &images,
        extent: capabilities.extent2d(),
    };

    let frames = memory::FramebufferList::new(&frames_cfg).expect("Failed to create framebuffers");

    cmd_buffer.begin_render_pass(&render_pass, &frames[img_index as usize]);

    cmd_buffer.bind_graphics_pipeline(&pipeline);

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

    #[allow(clippy::empty_loop)]
    loop { }
}