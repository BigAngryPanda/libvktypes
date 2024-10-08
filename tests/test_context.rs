#![allow(dead_code)]
use libvktypes::{
    libvk,
    dev,
    extensions,
    hw,
    layers,
    surface,
    window,
    swapchain,
    shader,
    graphics,
    memory,
    cmd
};

use std::sync::Once;
use std::mem::MaybeUninit;

static INIT_WINDOW: Once = Once::new();

static mut WINDOW: MaybeUninit<window::Window> = MaybeUninit::<window::Window>::uninit();

static INIT_GRAPHICS_INSTANCE: Once = Once::new();

static mut GRAPHICS_INSTANCE: MaybeUninit<libvk::Instance> = MaybeUninit::<libvk::Instance>::uninit();

static INIT_SURFACE: Once = Once::new();

static mut SURFACE: MaybeUninit<surface::Surface> = MaybeUninit::<surface::Surface>::uninit();

static INIT_GRAPHICS_HW: Once = Once::new();

static mut GRAPHICS_HW: MaybeUninit<hw::HWDevice> = MaybeUninit::<hw::HWDevice>::uninit();

static mut GRAPHICS_QUEUE: MaybeUninit<hw::QueueFamilyDescription> = MaybeUninit::<hw::QueueFamilyDescription>::uninit();

static INIT_PRESENT_QUEUE: Once = Once::new();

static mut PRESENT_QUEUE: MaybeUninit<hw::QueueFamilyDescription> = MaybeUninit::<hw::QueueFamilyDescription>::uninit();

static INIT_SURFACE_CAP: Once = Once::new();

static mut SURFACE_CAP: MaybeUninit<surface::Capabilities> = MaybeUninit::<surface::Capabilities>::uninit();

static INIT_GRAPHICS_DEV: Once = Once::new();

static mut GRAPHICS_DEV: MaybeUninit<dev::Device> = MaybeUninit::<dev::Device>::uninit();

static INIT_SWAPCHAIN: Once = Once::new();

static mut SWAPCHAIN: MaybeUninit<swapchain::Swapchain> = MaybeUninit::<swapchain::Swapchain>::uninit();

static INIT_VERT_SHADER: Once = Once::new();

static mut VERT_SHADER: MaybeUninit<shader::Shader> = MaybeUninit::<shader::Shader>::uninit();

static INIT_FRAG_SHADER: Once = Once::new();

static mut FRAG_SHADER: MaybeUninit<shader::Shader> = MaybeUninit::<shader::Shader>::uninit();

static INIT_RENDER_PASS: Once = Once::new();

static mut RENDER_PASS: MaybeUninit<graphics::RenderPass> = MaybeUninit::<graphics::RenderPass>::uninit();

static INIT_IMAGE_LIST: Once = Once::new();

static mut IMAGE_LIST: MaybeUninit<Vec<memory::ImageMemory>> = MaybeUninit::<Vec<memory::ImageMemory>>::uninit();

static INIT_CMD_POOL: Once = Once::new();

static mut CMD_POOL: MaybeUninit<cmd::Pool> = MaybeUninit::<cmd::Pool>::uninit();

static INIT_GRAPHICS_PIPELINE: Once = Once::new();

static mut GRAPHICS_PIPELINE: MaybeUninit<graphics::Pipeline> = MaybeUninit::<graphics::Pipeline>::uninit();

static INIT_FRAMEBUFFER: Once = Once::new();

static mut FRAMEBUFFER: MaybeUninit<Vec<memory::Framebuffer>> = MaybeUninit::<Vec<memory::Framebuffer>>::uninit();

pub fn get_window() -> &'static window::Window {
    unsafe {
        INIT_WINDOW.call_once(|| {
            WINDOW.write(window::create_window(
                &window::eventloop().expect("Failed to create eventloop")
            ).expect("Failed to create window"));
        });

        WINDOW.assume_init_ref()
    }
}

pub fn get_graphics_instance() -> &'static libvk::Instance {
    unsafe {
        INIT_GRAPHICS_INSTANCE.call_once(|| {
            let mut extensions = extensions::required_extensions(get_window());
            extensions.push(extensions::DEBUG_EXT_NAME);
            extensions.push(extensions::SURFACE_EXT_NAME);

            let lib_type = libvk::InstanceType {
                debug_layer: Some(layers::DebugLayer::default()),
                extensions: &extensions,
                ..libvk::InstanceType::default()
            };

            GRAPHICS_INSTANCE.write(libvk::Instance::new(&lib_type).expect("Failed to init graphic instance"));
        });

        GRAPHICS_INSTANCE.assume_init_ref()
    }
}

pub fn get_surface() -> &'static surface::Surface {
    unsafe {
        INIT_SURFACE.call_once(|| {
            SURFACE.write(surface::Surface::new(get_graphics_instance(), get_window()).expect("Failed to create surface"));
        });

        SURFACE.assume_init_ref()
    }
}

pub fn get_graphics_hw() -> &'static hw::HWDevice {
    unsafe {
        INIT_GRAPHICS_HW.call_once(|| {
            let surface = get_surface();
            let hw_list = hw::Description::poll(get_graphics_instance(), Some(surface)).expect("Failed to list hardware");

            let (hw_dev, qf, _) = hw_list
                .find_first(
                    hw::HWDevice::is_dedicated_gpu,
                    |q| q.is_graphics() && q.is_surface_supported(),
                    hw::any
                )
                .expect("Failed to find suitable hardware device");

            GRAPHICS_HW.write(hw_dev.clone());
            GRAPHICS_QUEUE.write(*qf);
        });

        GRAPHICS_HW.assume_init_ref()
    }
}

pub fn get_graphics_queue() -> &'static hw::QueueFamilyDescription {
    get_graphics_hw();

    unsafe { GRAPHICS_QUEUE.assume_init_ref() }
}

pub fn get_present_queue() -> &'static hw::QueueFamilyDescription {
    unsafe {
        INIT_PRESENT_QUEUE.call_once(|| {
            let hw = get_graphics_hw();
            let surface = get_surface();

            let present_queue = hw.find_first_queue(|q| q.support_surface(hw, surface));

            PRESENT_QUEUE.write(*present_queue.expect("Failed to find queue family with presentation capabilities"));
        });

        PRESENT_QUEUE.assume_init_ref()
    }
}

pub fn get_surface_capabilities() -> &'static surface::Capabilities {
    unsafe {
        INIT_SURFACE_CAP.call_once(|| {
            SURFACE_CAP.write(surface::Capabilities::get(get_graphics_hw(), get_surface()).expect("Failed to query capabilities"));
        });

        SURFACE_CAP.assume_init_ref()
    }
}

pub fn get_graphics_device() -> &'static dev::Device {
    unsafe {
        INIT_GRAPHICS_DEV.call_once(|| {
            let dev_type = dev::DeviceCfg {
                lib: get_graphics_instance(),
                hw: get_graphics_hw(),
                extensions: &[extensions::SWAPCHAIN_EXT_NAME],
                allocator: None,
            };

            GRAPHICS_DEV.write(dev::Device::new(&dev_type).expect("Failed to create device"));
        });

        GRAPHICS_DEV.assume_init_ref()
    }
}

pub fn get_swapchain() -> &'static swapchain::Swapchain {
    unsafe {
        INIT_SWAPCHAIN.call_once(|| {
            let lib_ref = get_graphics_instance();

            let surface_ref = get_surface();

            let device = get_graphics_device();

            let capabilities = get_surface_capabilities();

            let swp_type = swapchain::SwapchainCfg {
                num_of_images: 2,
                format: capabilities.formats().next().expect("No available formats").format,
                color: capabilities.formats().next().expect("No available formats").color_space,
                present_mode: *capabilities.modes().next().expect("No available modes"),
                flags: memory::UsageFlags::COLOR_ATTACHMENT,
                extent: capabilities.extent2d(),
                transform: capabilities.pre_transformation(),
                alpha: capabilities.alpha_composition(),
            };

            SWAPCHAIN.write(swapchain::Swapchain::new(lib_ref, device, surface_ref, &swp_type).expect("Failed to create swapchain"));
        });

        SWAPCHAIN.assume_init_ref()
    }
}

pub fn get_vert_shader() -> &'static shader::Shader {
    unsafe {
        INIT_VERT_SHADER.call_once(|| {
            let dev = get_graphics_device();

            let shader_type = shader::ShaderCfg {
                path: "tests/compiled_shaders/single_dot.spv",
                entry: "main",
            };

            VERT_SHADER.write(shader::Shader::from_file(dev, &shader_type).expect("Failed to create shader module"));
        });

        VERT_SHADER.assume_init_ref()
    }
}

pub fn get_frag_shader() -> &'static shader::Shader {
    unsafe {
        INIT_FRAG_SHADER.call_once(|| {
            let dev = get_graphics_device();

            let shader_type = shader::ShaderCfg {
                path: "tests/compiled_shaders/single_color.spv",
                entry: "main",
            };

            FRAG_SHADER.write(shader::Shader::from_file(dev, &shader_type).expect("Failed to create shader module"));
        });

        FRAG_SHADER.assume_init_ref()
    }
}

pub fn get_render_pass() -> &'static graphics::RenderPass {
    unsafe {
        INIT_RENDER_PASS.call_once(|| {
            let capabilities = get_surface_capabilities();

            let dev = get_graphics_device();

            RENDER_PASS.write(
                graphics::RenderPass::single_subpass(
                    dev,
                    capabilities.formats().next().expect("No available formats").format)
                    .expect("Failed to create render pass"));
        });

        RENDER_PASS.assume_init_ref()
    }
}

pub fn get_image_list() -> &'static Vec<memory::ImageMemory> {
    unsafe {
        INIT_IMAGE_LIST.call_once(|| {
            let swp = get_swapchain();

            IMAGE_LIST.write(swp.images().expect("Failed to get image list"));
        });

        IMAGE_LIST.assume_init_ref()
    }
}

pub fn get_cmd_pool() -> &'static cmd::Pool {
    unsafe {
        INIT_CMD_POOL.call_once(|| {
            let queue = get_graphics_queue();
            let dev = get_graphics_device();

            let pool_type = cmd::PoolCfg {
                queue_index: queue.index(),
            };

            CMD_POOL.write(cmd::Pool::new(dev, &pool_type).expect("Failed to allocate command pool"));
        });

        CMD_POOL.assume_init_ref()
    }
}

pub fn get_graphics_pipeline() -> &'static graphics::Pipeline {
    unsafe {
        INIT_GRAPHICS_PIPELINE.call_once(|| {
            let dev = get_graphics_device();
            let capabilities = get_surface_capabilities();

            let vertex_cfg = graphics::VertexInputCfg {
                location: 0,
                binding: 0,
                format: memory::ImageFormat::R32G32B32A32_SFLOAT,
                offset: 0,
            };

            let pipe_type = graphics::PipelineCfg {
                vertex_shader: get_vert_shader(),
                vertex_size: std::mem::size_of::<[f32; 2]>() as u32,
                vert_input: &[vertex_cfg],
                frag_shader: get_frag_shader(),
                geom_shader: None,
                topology: graphics::Topology::TRIANGLE_STRIP,
                extent: capabilities.extent2d(),
                push_constant_size: 0,
                render_pass: get_render_pass(),
                subpass_index: 0,
                enable_depth_test: false,
                enable_primitive_restart: false,
                cull_mode: graphics::CullMode::BACK,
                descriptor: &graphics::PipelineDescriptor::empty(dev)
            };

            GRAPHICS_PIPELINE.write(graphics::Pipeline::new(dev, &pipe_type).expect("Failed to create pipeline"));
        });

        GRAPHICS_PIPELINE.assume_init_ref()
    }
}

pub fn get_framebuffers() -> &'static Vec<memory::Framebuffer> {
    unsafe {
        INIT_FRAMEBUFFER.call_once(|| {
            let dev = get_graphics_device();

            let rp = get_render_pass();

            let imgs = get_image_list();

            let capabilities = get_surface_capabilities();

            let framebuffers: Vec<memory::Framebuffer> =
                imgs.iter().map(|img| {
                    let framebuffer_cfg = memory::FramebufferCfg {
                        render_pass: rp,
                        images: &[img.view(0)],
                        extent: capabilities.extent2d(),
                    };

                    memory::Framebuffer::new(dev, &framebuffer_cfg).expect("Failed to create framebuffer")
                }).collect();

            FRAMEBUFFER.write(
                framebuffers
            );
        });

        FRAMEBUFFER.assume_init_ref()
    }
}