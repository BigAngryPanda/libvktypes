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

use libvktypes::winit;

const VERT_SHADER: &str = "
#version 460

layout(location = 0) in vec4 position;

layout(set = 0, binding = 0) uniform Transformations {
    mat4 world;
    mat4 view;
    mat4 projection;
    mat4 scale;
    mat4 z_rotation;
    mat4 y_rotation;
} transformations;

void main() {
    vec4 projection =
        transformations.projection*
        transformations.view*
        transformations.world*
        transformations.y_rotation*
        transformations.z_rotation*
        transformations.scale*
        position;

    gl_Position = projection;
}
";

const FRAG_SHADER: &str = "
#version 460

layout(location = 0) out vec4 color;

layout(set = 0, binding = 1) uniform Colordata {
    vec4 data[6];
} colordata;

void main(){
    color = colordata.data[gl_PrimitiveID/2];
}
";

const VERTEX_DATA: &[f32] = &[
    -1.0, -1.0, -1.0, 1.0,
    -1.0, -1.0,  1.0, 1.0,

    1.0, -1.0, -1.0, 1.0,
    1.0, -1.0,  1.0, 1.0,

    -1.0, 1.0, -1.0, 1.0,
    -1.0, 1.0,  1.0, 1.0,

    1.0, 1.0, -1.0,  1.0,
    1.0, 1.0,  1.0,  1.0,
];

const INDICES: &[u32] = &[
    0, 1, 5,
    0, 5, 4,

    2, 1, 0,
    2, 3, 1,

    6, 4, 5,
    6, 5, 7,

    3, 2, 7,
    2, 6, 7,

    3, 7, 5,
    3, 5, 1,

    4, 2, 6,
    4, 0, 2,
];

const Y_ANGLE: f32 = std::f32::consts::FRAC_PI_4;

const COLOR_DATA: &[f32] = &[
    1.0, 0.0, 0.0, 1.0,
    0.0, 1.0, 0.0, 1.0,
    0.0, 0.0, 1.0, 1.0,
    0.5, 0.0, 1.0, 1.0,
    1.0, 0.5, 1.0, 1.0,
    1.0, 1.0, 0.5, 1.0,
];

const CAMERA_WIDTH: f32 = 3.0;

const CAMERA_HEIGTH: f32 = 3.0;

const CAMERA_NEAR_PLANE: f32 = 2.0;

const CAMERA_FAR_PLANE: f32 = 5.0;


const COEF_1: f32 = 2.0*CAMERA_NEAR_PLANE/CAMERA_WIDTH;
const COEF_2: f32 = 2.0*CAMERA_NEAR_PLANE/CAMERA_HEIGTH;
const COEF_3: f32 = CAMERA_FAR_PLANE/(CAMERA_FAR_PLANE - CAMERA_NEAR_PLANE);
const COEF_4: f32 = (-CAMERA_NEAR_PLANE*CAMERA_FAR_PLANE)/(CAMERA_FAR_PLANE - CAMERA_NEAR_PLANE);

fn main() {
    let mut z_angle: f32 = 0.0;

    let mut transformations = [
        // camera
/*
        0.0, 0.0, 1.0, 0.0,
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
*/
        // Move to the world space
         1.0, 0.0, 0.0, 0.0,
         0.0, 1.0, 0.0, 0.0,
         0.0, 0.0, 1.0, 0.0,
        -3.0, 0.0, 0.0, 1.0,

        // view
        0.0,  0.0, -1.0, 0.0,
        1.0,  0.0,  0.0, 0.0,
        0.0, -1.0,  0.0, 0.0,
        0.0,  0.0,  0.0, 1.0,

        // projection
        // a good explanation can be found here https://www.youtube.com/watch?v=U0_ONQQ5ZNM
        COEF_1, 0.0,    0.0,    0.0,
        0.0,    COEF_2, 0.0,    0.0,
        0.0,    0.0,    COEF_3, 1.0,
        0.0,    0.0,    COEF_4, 0.0,

        // scale
        0.25, 0.0,  0.0,  0.0,
        0.0,  0.25, 0.0,  0.0,
        0.0,  0.0,  0.25, 0.0,
        0.0,  0.0,  0.0,  1.0,

        // z rotation
        z_angle.cos(),  z_angle.sin(), 0.0, 0.0,
        -z_angle.sin(), z_angle.cos(), 0.0, 0.0,
        0.0,            0.0,           1.0, 0.0,
        0.0,            0.0,           0.0, 1.0,

        // y rotation
        Y_ANGLE.cos(), 0.0, -Y_ANGLE.sin(), 0.0,
        0.0,             1.0, 0.0,              0.0,
        Y_ANGLE.sin(), 0.0, Y_ANGLE.cos(),  0.0,
        0.0,             0.0, 0.0,              1.0
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

    let mem_cfg = memory::MemoryCfg {
        properties: hw::MemoryProperty::HOST_VISIBLE,
        filter: &hw::any,
        buffers: &[
            &memory::BufferCfg {
                size: std::mem::size_of_val(VERTEX_DATA) as u64,
                usage: memory::VERTEX,
                queue_families: &[queue.index()],
                simultaneous_access: false,
                count: 1
            },
            &memory::BufferCfg {
                size: std::mem::size_of_val(INDICES) as u64,
                usage: memory::INDEX,
                queue_families: &[queue.index()],
                simultaneous_access: false,
                count: 1
            },
            &memory::BufferCfg {
                size: std::mem::size_of_val(&transformations) as u64,
                usage: memory::UNIFORM,
                queue_families: &[queue.index()],
                simultaneous_access: false,
                count: 1
            },
            &memory::BufferCfg {
                size: std::mem::size_of_val(COLOR_DATA) as u64,
                usage: memory::UNIFORM,
                queue_families: &[queue.index()],
                simultaneous_access: false,
                count: 1
            }
        ]
    };

    let data = memory::Memory::allocate(&device, &mem_cfg).expect("Failed to allocate memory");

    data.access(&mut |bytes: &mut [f32]| {
        bytes.clone_from_slice(VERTEX_DATA);
    }, 0)
    .expect("Failed to fill the buffer");

    data.access(&mut |bytes: &mut [u32]| {
        bytes.clone_from_slice(INDICES);
    }, 1)
    .expect("Failed to fill indices");

    data.access(&mut |bytes: &mut [f32]| {
        bytes.clone_from_slice(&transformations);
    }, 2)
    .expect("Failed to fill coordinate transformations");

    data.access(&mut |bytes: &mut [f32]| {
        bytes.clone_from_slice(COLOR_DATA);
    }, 3)
    .expect("Failed to fill color data");

    let descs = graphics::PipelineDescriptor::allocate(&device, &[&[
        graphics::BindingCfg {
            resource_type: graphics::DescriptorType::UNIFORM_BUFFER,
            stage: graphics::ShaderStage::VERTEX,
            count: 1,
        },
        graphics::BindingCfg {
            resource_type: graphics::DescriptorType::UNIFORM_BUFFER,
            stage: graphics::ShaderStage::FRAGMENT,
            count: 1,
        }
    ]]).expect("Failed to allocate resources");

    descs.update(&[
        graphics::UpdateInfo {
            set: 0,
            binding: 0,
            starting_array_element: 0,
            resources: graphics::ShaderBinding::Buffers(&[data.view(2)]),
        },
        graphics::UpdateInfo {
            set: 0,
            binding: 1,
            starting_array_element: 0,
            resources: graphics::ShaderBinding::Buffers(&[data.view(3)]),
        },
    ]);

    let depth_buffer_cfg = [
        memory::ImageCfg {
            queue_families: &[queue.index()],
            simultaneous_access: false,
            format: memory::ImageFormat::D32_SFLOAT,
            extent: capabilities.extent3d(1),
            usage: memory::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            layout: memory::ImageLayout::UNDEFINED,
            aspect: memory::ImageAspect::DEPTH,
            tiling: memory::Tiling::OPTIMAL,
            count: 1
        }
    ];

    let alloc_info = memory::ImagesAllocationInfo {
        properties: hw::MemoryProperty::DEVICE_LOCAL,
        filter: &hw::any,
        image_cfgs: &depth_buffer_cfg
    };

    let depth_buffer = memory::ImageMemory::allocate(&device, &alloc_info).expect("Failed to allocate depth buffer");

    let render_pass = graphics::RenderPass::with_depth_buffers(&device, surf_format, memory::ImageFormat::D32_SFLOAT, 1)
        .expect("Failed to create render pass");

    let vertex_cfg = [
        graphics::VertexInputCfg {
            location: 0,
            binding: 0,
            format: memory::ImageFormat::R32G32B32A32_SFLOAT,
            offset: 0,
        }
    ];

    let pipe_type = graphics::PipelineCfg {
        vertex_shader: &vert_shader,
        vertex_size: std::mem::size_of::<[f32; 4]>() as u32,
        vert_input: &vertex_cfg,
        frag_shader: &frag_shader,
        geom_shader: None,
        topology: graphics::Topology::TRIANGLE_LIST,
        extent: capabilities.extent2d(),
        push_constant_size: 0,
        render_pass: &render_pass,
        subpass_index: 0,
        enable_depth_test: true,
        enable_primitive_restart: false,
        cull_mode: graphics::CullMode::BACK,
        descriptor: &descs,
    };

    let pipeline = graphics::Pipeline::new(&device, &pipe_type).expect("Failed to create pipeline");

    let img_sem = sync::Semaphore::new(&device).expect("Failed to create semaphore");
    let render_sem = sync::Semaphore::new(&device).expect("Failed to create semaphore");

    let cmd_pool_type = cmd::PoolCfg {
        queue_index: queue.index(),
    };

    let cmd_pool = cmd::Pool::new(&device, &cmd_pool_type).expect("Failed to allocate command pool");

    let images = swapchain.images().expect("Failed to get images");

    let frames: Vec<memory::Framebuffer> = images.iter()
        .map(|image| {
            let frames_cfg = memory::FramebufferCfg {
                render_pass: &render_pass,
                images: &[image.view(0), depth_buffer.view(0)],
                extent: capabilities.extent2d(),
            };

            memory::Framebuffer::new(&device, &frames_cfg).expect("Failed to create framebuffers")
        })
        .collect();

    let cmd_buffers: Vec<cmd::ExecutableBuffer> = frames.iter()
        .map(|frame| {
            let cmd_buffer = cmd_pool.allocate().expect("Failed to allocate command pool");

            cmd_buffer.begin_render_pass(&render_pass, &frame);
            cmd_buffer.bind_graphics_pipeline(&pipeline);
            cmd_buffer.bind_vertex_buffers(&[data.vertex_view(0, vertex_cfg[0].offset)]);
            cmd_buffer.bind_index_buffer(data.view(1), 0, memory::IndexBufferType::UINT32);
            cmd_buffer.bind_resources(&pipeline, &descs, &[]);
            cmd_buffer.draw_indexed(INDICES.len() as u32, 1, 0, 0, 0);
            cmd_buffer.end_render_pass();

            cmd_buffer.commit().expect("Failed to commit buffer")
        })
        .collect();

    let queue_cfg = queue::QueueCfg {
        family_index: queue.index(),
        queue_index: 0
    };

    let cmd_queue = queue::Queue::new(&device, &queue_cfg);

    event_loop.run(move |event, control_flow| {
        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => {
                control_flow.exit();
            },
            winit::event::Event::AboutToWait => {
                wnd.request_redraw();
            },
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::RedrawRequested,
                ..
            } => {
                z_angle += 0.01;

                transformations[64] = z_angle.cos();
                transformations[65] = z_angle.sin();
                transformations[68] = -z_angle.sin();
                transformations[69] = z_angle.cos();

                data.access(&mut |bytes: &mut [f32]| {
                    bytes.clone_from_slice(&transformations);
                }, 2)
                .expect("Failed to fill coordinate transformations");

                let img_index = swapchain.next_image(u64::MAX, Some(&img_sem), None).expect("Failed to get image index");

                let exec_info = queue::ExecInfo {
                    buffer: &cmd_buffers[img_index as usize],
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

                std::thread::sleep(std::time::Duration::from_millis(10));
            },
            _ => ()
        }

    }).expect("Failed to run example");
}
