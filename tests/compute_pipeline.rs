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

use std::ffi::CString;

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

    let mem_type = memory::MemoryCfg {
        size: 4,
        properties: hw::MemoryProperty::HOST_VISIBLE,
        usage: memory::BufferUsageFlags::STORAGE_BUFFER |
               memory::BufferUsageFlags::TRANSFER_SRC   |
               memory::BufferUsageFlags::TRANSFER_DST,
        shared_access: false,
        queue_families: &[queue.index()],
    };

    let selected_memory = device.find_memory(hw::any, &mem_type).expect("No suitable memory");

    let buff = memory::Memory::allocate(&device, &selected_memory, &mem_type).expect("Failed to allocate memory");

    let shader_type = shader::ShaderType {
        device: &device,
        path: "tests/compiled_shaders/fill_memory.spv",
        entry: CString::new("main").expect("Failed to allocate string"),
    };

    let shader = shader::Shader::from_file(&shader_type).expect("Failed to create shader module");

    let pipe_type = compute::PipelineType {
        device: &device,
        buffers: &[&buff],
        shader: &shader,
        push_constant_size: 0,
    };

    assert!(compute::Pipeline::new(&pipe_type).is_ok());
}