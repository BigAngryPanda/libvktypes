#[path = "./mod.rs"]
mod test_context;

use libvktypes::{
    dev,
    extensions,
    hw,
    layers,
    libvk,
    shader,
};

#[test]
fn load_shader() {
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

    let shader_type = shader::ShaderCfg {
        path: "tests/compiled_shaders/fill_memory.spv",
        entry: "main",
    };

    assert!(shader::Shader::from_file(&device, &shader_type).is_ok());
}

#[test]
fn from_glsl() {
    let device = test_context::get_graphics_device();

    let shader_type = shader::ShaderCfg {
        path: "tests/shaders/single_dot.vert",
        entry: "main",
    };

    assert!(shader::Shader::from_glsl_file(&device, &shader_type, shader::Kind::Vertex).is_ok());
}