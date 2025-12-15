mod test_context;

mod cmd {
    use libvktypes::{
        dev,
        extensions,
        hw,
        layers,
        libvk,
        memory,
        shader,
        compute,
        cmd,
        queue,
        formats,
        graphics
    };

    use libvktypes::memory::BufferView;

    use super::test_context;

    fn cmd_pool_allocation() {
        let lib_type = libvk::InstanceType {
            debug_layer: Some(layers::DebugLayer::default()),
            extensions: &[extensions::DEBUG_EXT_NAME],
            version_major: 1,
            version_minor: 3,
            version_patch: 0,
            ..libvk::InstanceType::default()
        };

        let lib = libvk::Instance::new(&lib_type).expect("Failed to load library");
        let hw_list = hw::Description::poll(&lib, None).expect("Failed to list hardware");

        let (hw_dev, _, _) = hw_list
            .find_first(
                hw::HWDevice::is_dedicated_gpu,
                hw::QueueFamilyDescription::is_compute,
                |_| true
            )
            .expect("Failed to find suitable hardware device");

        let dev_type = dev::DeviceCfg {
            lib: &lib,
            hw: hw_dev,
            extensions: &[],
            allocator: None,
        };

        let device = dev::Device::new(&dev_type).expect("Failed to create device");

        assert!(cmd::Pool::new(&device, 0).is_ok());
    }

    fn cmd_buffer_exec() {
        let lib_type = libvk::InstanceType {
            debug_layer: Some(layers::DebugLayer::default()),
            extensions: &[extensions::DEBUG_EXT_NAME],
            version_major: 1,
            version_minor: 3,
            version_patch: 0,
            ..libvk::InstanceType::default()
        };

        let lib = libvk::Instance::new(&lib_type).expect("Failed to load library");
        let hw_list = hw::Description::poll(&lib, None).expect("Failed to list hardware");

        let (hw_dev, queue, _) = hw_list
            .find_first(
                hw::HWDevice::is_dedicated_gpu,
                hw::QueueFamilyDescription::is_compute,
                |_| true
            )
            .expect("Failed to find suitable hardware device");

        let dev_type = dev::DeviceCfg {
            lib: &lib,
            hw: hw_dev,
            extensions: &[],
            allocator: None,
        };

        let device = dev::Device::new(&dev_type).expect("Failed to create device");

        let mem_cfg = [
            memory::layout::LayoutElementCfg::Buffer {
                size: 4,
                usage: memory::STORAGE,
                queue_families: &[queue.index()],
                simultaneous_access: false,
                count: 1
            }
        ];

        let storage = memory::Memory::allocate_host_coherent_memory(
            &device, &mut mem_cfg.iter())
        .expect("Failed to allocate memory");

        let compute_buffer = memory::RefView::new(&storage, 0);

        let shader_type = shader::ShaderCfg {
            path: "tests/compiled_shaders/fill_memory.spv",
            entry: "main",
        };

        let shader = shader::Shader::from_file(&device, &shader_type).expect("Failed to create shader module");

        let pipe_type = compute::PipelineCfg {
            buffers: &[compute_buffer],
            shader: &shader,
            push_constant_size: 0,
        };

        let pipeline = compute::Pipeline::new(&device, &pipe_type).expect("Failed to create pipeline");

        let cmd_pool = cmd::Pool::new(&device, queue.index()).expect("Failed to allocate command pool");

        let cmd_buffer = cmd_pool.allocate().expect("Failed to allocate command buffer");

        cmd_buffer.bind_compute_pipeline(&pipeline);

        cmd_buffer.dispatch(1, 1, 1);

        let exec_buffer = cmd_buffer.commit().expect("Failed to commit command buffer");

        let queue_type = queue::QueueCfg {
            family_index: queue.index(),
            queue_index: 0,
        };

        let queue = queue::Queue::new(&device, &queue_type);

        let exec_info = queue::ExecInfo {
            wait_stage: cmd::PipelineStage::COMPUTE_SHADER,
            buffer: &exec_buffer,
            timeout: u64::MAX,
            wait: &[],
            signal: &[],
        };

        assert!(queue.exec(&exec_info).is_ok())
    }

    fn write_graphics_cmds() {
        let pipeline = test_context::get_graphics_pipeline();

        let framebuffers = &test_context::get_framebuffers();

        let pool = test_context::get_cmd_pool();

        let render_pass = test_context::get_render_pass();

        let cmd_buffer = pool.allocate().expect("Failed to allocate cmd buffer");

        cmd_buffer.begin_render_pass(&render_pass, &framebuffers[0]);

        cmd_buffer.bind_graphics_pipeline(&pipeline);

        cmd_buffer.end_render_pass();

        assert!(cmd_buffer.commit().is_ok());
    }

    fn copy_to_image_buffer() {
        let queue = test_context::get_graphics_queue();

        let device = test_context::get_graphics_device();

        let format = memory::ImageFormat::R8G8B8A8_SRGB;

        let compute_memory = [memory::layout::LayoutElementCfg::Buffer {
                size: 800*600*formats::block_size(format),
                usage: memory::BufferUsageFlags::TRANSFER_SRC,
                queue_families: &[queue.index()],
                simultaneous_access: false,
                count: 1
            }
        ];

        let staging_storage = memory::Memory::allocate_host_memory(
            &device, &mut compute_memory.iter())
        .expect("Failed to allocate memory");

        let staging_buffer = memory::RefView::new(&staging_storage, 0);

        staging_buffer.access(&mut |bytes: &mut [u8]| {
            bytes.fill(0x42);
        }).expect("Failed to write to the staging buffer");

        let image_cfg = [memory::layout::LayoutElementCfg::Image {
                queue_families: &[queue.index()],
                simultaneous_access: false,
                format: format,
                extent: memory::Extent3D {height: 800, width: 600, depth: 1 },
                usage: memory::ImageUsageFlags::COLOR_ATTACHMENT | memory::ImageUsageFlags::TRANSFER_DST,
                layout: memory::ImageLayout::UNDEFINED,
                aspect: memory::ImageAspect::COLOR,
                tiling: memory::Tiling::OPTIMAL,
                count: 1
            }
        ];

        let image_storage =
            memory::Memory::allocate_device_memory(&device, &mut image_cfg.iter())
            .expect("Failed to allocate image memory");

        let dst_image = memory::RefImageView::new(&image_storage, 0);

        let pool = test_context::get_cmd_pool();

        let cmd_buffer = pool.allocate().expect("Failed to allocate cmd buffer");

        cmd_buffer.set_image_barrier(
            dst_image,
            cmd::AccessType::empty(),
            cmd::AccessType::TRANSFER_WRITE,
            memory::ImageLayout::UNDEFINED,
            memory::ImageLayout::TRANSFER_DST_OPTIMAL,
            graphics::PipelineStage::TOP_OF_PIPE,
            graphics::PipelineStage::TRANSFER,
            cmd::QUEUE_FAMILY_IGNORED,
            cmd::QUEUE_FAMILY_IGNORED);

        cmd_buffer.copy_buffer_to_image(staging_buffer, dst_image);

        cmd_buffer.set_image_barrier(
            dst_image,
            cmd::AccessType::TRANSFER_WRITE,
            cmd::AccessType::MEMORY_READ,
            memory::ImageLayout::TRANSFER_DST_OPTIMAL,
            memory::ImageLayout::GENERAL,
            graphics::PipelineStage::TRANSFER,
            graphics::PipelineStage::BOTTOM_OF_PIPE,
            cmd::QUEUE_FAMILY_IGNORED,
            cmd::QUEUE_FAMILY_IGNORED);

        let exec_buffer = cmd_buffer.commit().expect("Failed to commit command buffer");

        let queue_type = queue::QueueCfg {
            family_index: queue.index(),
            queue_index: 0,
        };

        let queue = queue::Queue::new(&device, &queue_type);

        let exec_info = queue::ExecInfo {
            wait_stage: cmd::PipelineStage::COMPUTE_SHADER,
            buffer: &exec_buffer,
            timeout: u64::MAX,
            wait: &[],
            signal: &[],
        };

        assert!(queue.exec(&exec_info).is_ok())
    }

    #[test]
    fn tests() {
        cmd_pool_allocation();
        cmd_buffer_exec();
        copy_to_image_buffer();
        write_graphics_cmds();
    }
}

