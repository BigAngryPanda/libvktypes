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
        pipeline
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

        let mem_cfg = [memory::LayoutElementCfg::Buffer {
            size: 1,
            usage: memory::STORAGE,
            queue_families: &[queue.index()],
            simultaneous_access: false,
            count: 1
        }];

        let data = memory::Memory::allocate_host_memory(&device, &mut mem_cfg.iter()).expect("Failed to allocate memory");

        let view = memory::view::RefView::new(&data, 0);

        let shader = shader::ShaderBuilder::new()
            .path("tests/compiled_shaders/fill_memory.spv")
            .from_file(&device)
            .expect("Failed to create shader module");

        let layout = pipeline::PipelineLayoutBuilder::with_sets(1)
            .binding(0, 0, pipeline::ShaderStage::COMPUTE, pipeline::DescriptorType::STORAGE_BUFFER, 1)
            .build(&device)
            .expect("Failed to crate pipeline layout");

        let bindings = pipeline::PipelineBindings::new(&device, &layout).expect("Failed to create bindings");

        let mut write_info = pipeline::WriteInfo::new();
        write_info
            .buffer(0, 0, pipeline::DescriptorType::STORAGE_BUFFER)
            .element(view);

        bindings.write(&write_info);

        let pipe = pipeline::ComputePipelineBuilder::new()
            .compute_shader(&shader)
            .build(&device, &layout);

        assert!(pipe.is_ok());
    }
}