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
    queue,
    pipeline
};

use libvktypes::memory::BufferView;

use std::mem::{
    size_of,
    size_of_val
};

const VERT_SHADER: &str = "
#version 460

layout (location = 0) in vec4 pos;
layout (location = 1) in vec2 in_uv;

layout (location = 0) out vec2 out_uv;

void main() {
    out_uv = in_uv;
    gl_Position = pos;
}
";

const FRAG_SHADER: &str = "
#version 460

layout (location = 0) in vec2 uv;
layout (location = 0) out vec4 out_color;

layout (set = 0, binding = 0) uniform sampler2D samplerColor;

void main() {
    out_color = texture(samplerColor, uv);
}
";

const TEXTURE_WIDTH: u32  = 3;
const TEXTURE_HEIGHT: u32 = 2;

const TEXTURE_SIZE: usize = (TEXTURE_WIDTH*TEXTURE_HEIGHT) as usize;

const TEXTURE_DATA: [u32; TEXTURE_SIZE] = [
    0x000000FF, 0x00000000, 0x0000FF00,
    0x00FF0000, 0x00FFFFFF, 0x00FFFF00
];

const VERTEX_DATA: &[f32] = &[
    -0.8, -0.8, 0.0, 1.0, 0.0, 0.0, // top left corner
    -0.8,  0.8, 0.0, 1.0, 0.0, 1.0, // bottom left
     0.8,  0.8, 0.0, 1.0, 1.0, 1.0, // bottom right
     0.8, -0.8, 0.0, 1.0, 1.0, 0.0, // top right
];

const INDICES: &[u32] = &[
    0, 1, 2,
    0, 2, 3
];

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

    let cmd_pool = cmd::Pool::new(&device, queue.index()).expect("Failed to allocate command pool");

    let copy_cmd_queue = cmd_pool.allocate().expect("Failed to allocate command pool");

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
        .path("VERT_DATA")
        .glsl_src(VERT_SHADER)
        .shader_type(shader::Kind::Vertex)
        .from_glsl(&device)
        .expect("Failed to create vertex shader module");

    let frag_shader = shader::ShaderBuilder::new()
        .path("FRAG_DATA")
        .glsl_src(FRAG_SHADER)
        .shader_type(shader::Kind::Fragment)
        .from_glsl(&device)
        .expect("Failed to create vertex shader module");

    let buffers = [
        memory::LayoutElementCfg::Buffer {
            size: size_of_val(VERTEX_DATA) as u64,
            usage: memory::VERTEX,
            queue_families: &[queue.index()],
            simultaneous_access: false,
            count: 1
        },
        memory::LayoutElementCfg::Buffer {
            size: size_of_val(INDICES) as u64,
            usage: memory::INDEX,
            queue_families: &[queue.index()],
            simultaneous_access: false,
            count: 1
        },
        memory::LayoutElementCfg::Buffer {
            size: (TEXTURE_SIZE*size_of::<u32>()) as u64,
            usage: memory::BufferUsageFlags::TRANSFER_SRC,
            queue_families: &[queue.index()],
            simultaneous_access: false,
            count: 1
        }
    ];

    let data = memory::Memory::allocate_host_memory(&device, &mut buffers.iter()).expect("Failed to allocate memory");

    let vertices = memory::RefView::new(&data, 0);
    let indices  = memory::RefView::new(&data, 1);
    let texture_stage = memory::RefView::new(&data, 2);

    vertices.access(&mut |bytes: &mut [f32]| {
        bytes.clone_from_slice(VERTEX_DATA);
    }).expect("Failed to fill vertex buffer");

    indices.access(&mut |bytes: &mut [u32]| {
        bytes.clone_from_slice(INDICES);
    }).expect("Failed to fill index buffer");

    texture_stage.access(&mut |bytes: &mut [u32]| {
        bytes.clone_from_slice(&TEXTURE_DATA);
    }).expect("Failed to fill texture buffer");

    let image_cfgs = [
        memory::LayoutElementCfg::Image {
            queue_families: &[queue.index()],
            simultaneous_access: false,
            format: memory::ImageFormat::R8G8B8A8_SRGB,
            extent: memory::Extent3D {width: TEXTURE_WIDTH, height: TEXTURE_HEIGHT, depth: 1},
            usage:  memory::ImageUsageFlags::SAMPLED | memory::ImageUsageFlags::TRANSFER_DST,
            layout: memory::ImageLayout::UNDEFINED,
            aspect: memory::ImageAspect::COLOR,
            tiling: memory::Tiling::OPTIMAL,
            count: 1,
            sample_count: memory::SampleCountFlags::TYPE_1
        }
    ];

    let texture_memory =
        memory::Memory::allocate_device_memory(&device, &mut image_cfgs.iter()).expect("Failed to allocate texture memory");

    let texture = memory::view::RefImageView::new(&texture_memory, 0);

    copy_cmd_queue.set_image_barrier(
        texture,
        cmd::AccessType::NONE,
        cmd::AccessType::TRANSFER_WRITE,
        memory::ImageLayout::UNDEFINED,
        memory::ImageLayout::TRANSFER_DST_OPTIMAL,
        graphics::PipelineStage::BOTTOM_OF_PIPE,
        graphics::PipelineStage::TRANSFER,
        cmd::QUEUE_FAMILY_IGNORED,
        cmd::QUEUE_FAMILY_IGNORED
    );

    copy_cmd_queue.copy_buffer_to_image(texture_stage, texture);

    copy_cmd_queue.set_image_barrier(
        texture,
        cmd::AccessType::TRANSFER_WRITE,
        cmd::AccessType::SHADER_READ,
        memory::ImageLayout::TRANSFER_DST_OPTIMAL,
        memory::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        graphics::PipelineStage::TRANSFER,
        graphics::PipelineStage::FRAGMENT_SHADER,
        cmd::QUEUE_FAMILY_IGNORED,
        cmd::QUEUE_FAMILY_IGNORED
    );

    let cmd_queue = queue::Queue::new(&device, queue.index(), 0);

    let fence = sync::Fence::new(&device, false).expect("Failed to create fence");

    let copy_exec_info = queue::ExecInfo {
        buffer: &copy_cmd_queue.commit().expect("Failed to commit buffer"),
        wait_stage: cmd::PipelineStage::COLOR_ATTACHMENT_OUTPUT,
        timeout: u64::MAX,
        wait: &[],
        signal: &[],
        fence: &fence
    };

    cmd_queue.exec_with_fence(&copy_exec_info, false, u64::MAX).expect("Failed to copy texture");

    let sampler_cfg = graphics::SamplerCfg {
        address_mode_u: graphics::SamplerAddressMode::MIRRORED_REPEAT,
        address_mode_v: graphics::SamplerAddressMode::MIRRORED_REPEAT,
        ..Default::default()
    };

    let sampler = graphics::Sampler::new(&device, &sampler_cfg).expect("Failed to create sampler");

    let render_pass = graphics::RenderPass::single_subpass(&device, surf_format)
        .expect("Failed to create render pass");

    let layout = pipeline::PipelineLayoutBuilder::with_sets(1)
        .binding(0, 0, graphics::ShaderStage::FRAGMENT,
            pipeline::DescriptorType::COMBINED_IMAGE_SAMPLER, 1)
        .build(&device)
        .expect("Failed to crate pipeline layout");

    let bindings = pipeline::PipelineBindings::new(&device, &layout).expect("Failed to create bindings");

    let mut write_info = pipeline::WriteInfo::new();
    write_info
        .image(0, 0, pipeline::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .element(texture, &sampler, memory::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

    bindings.write(&write_info);

    let pipeline = pipeline::GraphicsPipelineBuilder::new()
        .vertex_shader(&vert_shader)
        .vertex_binding_input(0, std::mem::size_of::<[f32; 6]>() as u32)
        .vertex_input(0, 0, memory::ImageFormat::R32G32B32A32_SFLOAT, 0)
        .vertex_input(1, 0, memory::ImageFormat::R32G32_SFLOAT, size_of::<[f32; 4]>() as u32)
        .frag_shader(&frag_shader)
        .render_pass(&render_pass)
        .extent2d(capabilities.extent2d())
        .build(&device, &layout)
        .expect("failed to create pipeline");

    let img_sem = sync::Semaphore::new(&device).expect("Failed to create semaphore");
    let render_sem = sync::Semaphore::new(&device).expect("Failed to create semaphore");

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

    cmd_buffer.bind_vertex_buffers_with_offsets(&[
        (vertices, 0),
        (vertices, size_of::<[f32; 4]>() as u64)]);

    cmd_buffer.bind_index_buffer(indices, 0, memory::IndexBufferType::UINT32);

    cmd_buffer.bind_resources(&layout, &bindings, &[]);

    cmd_buffer.draw_indexed(INDICES.len() as u32, 1, 0, 0, 0);

    cmd_buffer.end_render_pass();

    let exec_buffer = cmd_buffer.commit().expect("Failed to commit buffer");

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