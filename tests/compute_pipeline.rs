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
            //|dev| hw::HWDevice::is_discrete_gpu(dev) || hw::HWDevice::is_integrated_gpu(dev),
            hw::HWDevice::is_discrete_gpu,
            hw::QueueFamilyDescription::is_compute,
            |_| true
        )
        .expect("Failed to find suitable hardware device");

    let dev_type = dev::DeviceCfg {
        lib: &lib,
        hw: hw_dev,
        queue_family_index: queue.index(),
        priorities: &[1.0_f32],
        extensions: &[],
        allocator: None,
    };

    let device = dev::Device::new(&dev_type).expect("Failed to create device");

    let mem_type = memory::MemoryType {
        device: &device,
        size: 1,
        properties: hw::MemoryProperty::HOST_VISIBLE,
        usage: memory::BufferUsageFlags::STORAGE_BUFFER |
               memory::BufferUsageFlags::TRANSFER_SRC   |
               memory::BufferUsageFlags::TRANSFER_DST,
        sharing_mode: memory::SharingMode::EXCLUSIVE,
        queue_families: &[device.queue_index()],
    };

    let buff = memory::Memory::allocate(&mem_type).expect("Failed to allocate memory");

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