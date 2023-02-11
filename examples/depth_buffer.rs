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

    //assert!(capabilities.is_img_count_supported(2));
    assert!(capabilities.is_format_supported(surface::SurfaceFormat {
        format: surface::ImageFormat::B8G8R8A8_UNORM,
        color_space: surface::ColorSpace::SRGB_NONLINEAR,
    }));
    assert!(capabilities.is_mode_supported(surface::PresentMode::FIFO));
    assert!(capabilities.is_flags_supported(surface::UsageFlags::COLOR_ATTACHMENT));

    let swp_type = swapchain::SwapchainType {
        lib: &lib,
        dev: &device,
        surface: &surface,
        num_of_images: 2,
        format: surface::ImageFormat::B8G8R8A8_UNORM,
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
        path: "examples/compiled_shaders/depth_buffer.vert.spv",
        entry: CString::new("main").expect("Failed to allocate string"),
    };

    let vert_shader = shader::Shader::from_file(&vert_shader_type).expect("Failed to create vertex shader module");

    let frag_shader_type = shader::ShaderType {
        device: &device,
        path: "examples/compiled_shaders/depth_buffer.frag.spv",
        entry: CString::new("main").expect("Failed to allocate string"),
    };

    let frag_shader = shader::Shader::from_file(&frag_shader_type).expect("Failed to create fragment shader module");

    let surf_format = capabilities.formats().next().expect("No available formats").format;

    let mem_type = memory::MemoryType {
        device: &device,
        size: 16*7,
        properties: hw::MemoryProperty::HOST_VISIBLE | hw::MemoryProperty::HOST_COHERENT,
        usage: memory::BufferUsageFlags::VERTEX_BUFFER |
               memory::BufferUsageFlags::TRANSFER_SRC  |
               memory::BufferUsageFlags::TRANSFER_DST,
        sharing_mode: memory::SharingMode::EXCLUSIVE,
        queue_families: &[queue.index()],
    };

    let vertex_data = memory::Memory::allocate(&mem_type).expect("Failed to allocate memory");

    let mut set_vrtx_buffer = |bytes: &mut [f32]| {
        bytes.clone_from_slice(&[0.5f32, 0.5f32, 0.0f32, 1.0f32,
                     0.5f32, -0.5f32, 0.0f32, 1.0f32,
                     -0.5f32, 0.5f32, 0.0f32, 1.0f32,
                     1.0f32, 0.0f32, 1.0f32, 1.0f32,
                     -1.0f32, 0.0f32, 1.0f32, 1.0f32,
                     1.0f32, 1.0f32, 1.0f32, 1.0f32,
                     -1.0f32, 1.0f32, 1.0f32, 1.0f32,]);
    };

    vertex_data.write(&mut set_vrtx_buffer).expect("Failed to fill the buffer");

    let depth_type = memory::ImageType {
        device: &device,
        queue_families: &[queue.index()],
        format: surface::ImageFormat::D32_SFLOAT,
        extent: capabilities.extent3d(1),
        usage: memory::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        layout: graphics::ImageLayout::UNDEFINED,
        aspect: memory::ImageAspect::DEPTH,
        properties: hw::MemoryProperty::DEVICE_LOCAL,
    };

    let depth_buffer = memory::Image::new(&depth_type).expect("Failed to allocate depth buffer");

    let subpass_info = [
        graphics::SubpassInfo {
            input_attachments: &[],
            color_attachments: &[0],
            resolve_attachments: &[],
            depth_stencil_attachment: 1,
            preserve_attachments: &[],
        }
    ];

    let attachments = [
        graphics::AttachmentInfo {
            format: surf_format,
            load_op: graphics::AttachmentLoadOp::CLEAR,
            store_op: graphics::AttachmentStoreOp::STORE,
            stencil_load_op: graphics::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: graphics::AttachmentStoreOp::DONT_CARE,
            initial_layout: graphics::ImageLayout::UNDEFINED,
            final_layout: graphics::ImageLayout::PRESENT_SRC_KHR,
        },
        graphics::AttachmentInfo {
            format: surface::ImageFormat::D32_SFLOAT,
            load_op: graphics::AttachmentLoadOp::CLEAR,
            store_op: graphics::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: graphics::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: graphics::AttachmentStoreOp::DONT_CARE,
            initial_layout: graphics::ImageLayout::UNDEFINED,
            final_layout: graphics::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        }
    ];

    let subpass_sync_info = [
        graphics::SubpassSync {
            src_subpass: graphics::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage: graphics::PipelineStage::BOTTOM_OF_PIPE,
            dst_stage: graphics::PipelineStage::COLOR_ATTACHMENT_OUTPUT,
            src_access: graphics::AccessFlags::MEMORY_READ,
            dst_access: graphics::AccessFlags::COLOR_ATTACHMENT_WRITE | graphics::AccessFlags::COLOR_ATTACHMENT_READ,
        },
        graphics::SubpassSync {
            src_subpass: 0,
            dst_subpass: graphics::SUBPASS_EXTERNAL,
            src_stage: graphics::PipelineStage::COLOR_ATTACHMENT_OUTPUT,
            dst_stage: graphics::PipelineStage::BOTTOM_OF_PIPE,
            src_access: graphics::AccessFlags::COLOR_ATTACHMENT_WRITE | graphics::AccessFlags::COLOR_ATTACHMENT_READ,
            dst_access: graphics::AccessFlags::MEMORY_READ,
        }
    ];

    let rp_cfg = graphics::RenderPassCfg {
        attachments: &attachments,
        sync_info: &subpass_sync_info,
        subpasses: &subpass_info,
    };

    let render_pass = graphics::RenderPass::new(&device, &rp_cfg)
        .expect("Failed to create render pass");

    let pipe_type = graphics::PipelineType {
        device: &device,
        vertex_shader: &vert_shader,
        vertex_size: std::mem::size_of::<[f32; 4]>() as u32,
        vert_slots: 1,
        vert_input: &[graphics::VertexInputCfg {
            location: 0,
            binding: 0,
            format: surface::ImageFormat::R32G32B32A32_SFLOAT,
            offset: 0,
        }],
        frag_shader: &frag_shader,
        topology: graphics::Topology::TRIANGLE_STRIP,
        extent: capabilities.extent2d(),
        push_constant_size: 0,
        render_pass: &render_pass,
        subpass_index: 0,
        enable_depth: true,
    };

    let pipeline = graphics::Pipeline::new(&pipe_type).expect("Failed to create pipeline");

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

    let framebuffer_cfg = memory::FramebufferType {
        device: &device,
        images: &[&images[img_index as usize], &depth_buffer],
        extent: capabilities.extent2d(),
        render_pass: &render_pass,
    };

    let framebuffer = memory::Framebuffer::new(&framebuffer_cfg).expect("Failed to create framebuffer");

    cmd_buffer.begin_render_pass(&render_pass, &framebuffer);

    cmd_buffer.bind_graphics_pipeline(&pipeline);

    let vrtx_stage_data = [&vertex_data];

    cmd_buffer.bind_vertex_buffers(&vrtx_stage_data);

    cmd_buffer.draw(7, 1, 0, 0);

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