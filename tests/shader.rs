use libvktypes::{
    dev,
    extensions,
    hw,
    layers,
    libvk,
    shader,
};

use std::ffi::CString;

#[test]
fn load_shader() {
    let lib_type = libvk::InstanceType {
        debug_layer: Some(layers::DebugLayer::default()),
        extensions: &[extensions::DEBUG_EXT_NAME],
        ..libvk::InstanceType::default()
    };

    let lib = libvk::Instance::new(&lib_type).expect("Failed to load library");
    let hw_list = hw::Description::poll(&lib).expect("Failed to list hardware");

    let (hw_dev, queue, _) = hw_list
        .find_first(
            //|dev| hw::HWDevice::is_discrete_gpu(dev) || hw::HWDevice::is_integrated_gpu(dev),
            hw::HWDevice::is_discrete_gpu,
            hw::QueueFamilyDescription::is_compute,
            |_| true,
        )
        .expect("Failed to find suitable hardware device");

    let dev_type = dev::DeviceType {
        lib: &lib,
        hw: hw_dev,
        queue_family_index: queue.index(),
        priorities: &[1.0_f32],
        extensions: &[],
    };

    let device = dev::Device::new(&dev_type).expect("Failed to create device");

    let shader_type = shader::ShaderType {
        device: &device,
        path: "tests/compiled_shaders/fill_memory.spv",
        entry: CString::new("main").expect("Failed to allocate string"),
    };

    assert!(shader::Shader::from_file(&shader_type).is_ok());
}