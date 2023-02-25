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
        queue
    };

    use super::test_context;

    #[test]
    fn cmd_pool_allocation() {
        let lib_type = libvk::InstanceType {
            debug_layer: Some(layers::DebugLayer::default()),
            extensions: &[extensions::DEBUG_EXT_NAME],
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

        let cmd_pool_type = cmd::PoolCfg {
            queue_index: 0,
        };

        assert!(cmd::Pool::new(&device, &cmd_pool_type).is_ok());
    }

    #[test]
    fn cmd_buffer_exec() {
        let lib_type = libvk::InstanceType {
            debug_layer: Some(layers::DebugLayer::default()),
            extensions: &[extensions::DEBUG_EXT_NAME],
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

        let mem_type = memory::StorageCfg {
            size: 4,
            properties: hw::MemoryProperty::HOST_VISIBLE | hw::MemoryProperty::HOST_COHERENT | hw::MemoryProperty::HOST_CACHED,
            usage: memory::BufferUsageFlags::STORAGE_BUFFER |
                memory::BufferUsageFlags::TRANSFER_SRC   |
                memory::BufferUsageFlags::TRANSFER_DST,
            shared_access: false,
            queue_families: &[queue.index()],
        };

        let selected_memory = device.find_memory(hw::any, &mem_type).expect("No suitable memory");

        let buff = memory::Storage::allocate(&device, &selected_memory, &mem_type).expect("Failed to allocate memory");

        let shader_type = shader::ShaderCfg {
            path: "tests/compiled_shaders/fill_memory.spv",
            entry: "main",
        };

        let shader = shader::Shader::from_file(&device, &shader_type).expect("Failed to create shader module");

        let pipe_type = compute::PipelineCfg {
            buffers: &[&buff],
            shader: &shader,
            push_constant_size: 0,
        };

        let pipeline = compute::Pipeline::new(&device, &pipe_type).expect("Failed to create pipeline");

        let cmd_pool_type = cmd::PoolCfg {
            queue_index: queue.index(),
        };

        let cmd_pool = cmd::Pool::new(&device, &cmd_pool_type).expect("Failed to allocate command pool");

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

    #[test]
    fn write_graphics_cmds() {
        let render_pass = test_context::get_render_pass();

        let pipeline = test_context::get_graphics_pipeline();

        let framebuffer = &test_context::get_framebuffers()[0];

        let pool = test_context::get_cmd_pool();

        let cmd_buffer = pool.allocate().expect("Failed to allocate cmd buffer");

        cmd_buffer.begin_render_pass(render_pass, framebuffer);

        cmd_buffer.bind_graphics_pipeline(pipeline);

        cmd_buffer.end_render_pass();

        assert!(cmd_buffer.commit().is_ok());
    }
}