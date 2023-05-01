#[cfg(test)]
mod compute_pipeline {
    use libvktypes::{
        dev,
        extensions,
        hw,
        layers,
        libvk,
        memory,
        shader,
        compute,
    };

    #[test]
    fn create_pipeline() {
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

        let compute_memory = memory::BufferCfg {
            size: 1,
            usage: memory::STORAGE,
            queue_families: &[queue.index()],
            simultaneous_access: false,
            count: 1
        };

        let mem_cfg = memory::MemoryCfg {
            properties: hw::MemoryProperty::HOST_VISIBLE,
            filter: &hw::any,
            buffers: &[&compute_memory]
        };

        let data = memory::Memory::allocate(&device, &mem_cfg).expect("Failed to allocate memory");

        let shader_type = shader::ShaderCfg {
            path: "tests/compiled_shaders/fill_memory.spv",
            entry: "main",
        };

        let shader = shader::Shader::from_file(&device, &shader_type).expect("Failed to create shader module");

        let pipe_type = compute::PipelineCfg {
            buffers: &[data.view(0)],
            shader: &shader,
            push_constant_size: 0,
        };

        assert!(compute::Pipeline::new(&device, &pipe_type).is_ok());
    }
}