use libvktypes::resources::{
    libvk,
    layers,
    extensions
};

#[test]
fn default_instance() {
    let lib = libvk::Instance::new(&libvk::InstanceType::default());

    assert!(lib.is_ok());
}

#[test]
fn debug_instance() {
    let lib_type = libvk::InstanceType {
        debug_layer: Some(layers::DebugLayer::default()),
        extensions: &[extensions::DEBUG_EXT_NAME],
        ..libvk::InstanceType::default()
    };

    let lib = libvk::Instance::new(&lib_type);

    assert!(lib.is_ok());
}

#[test]
fn dynamic_load_instance() {
    let lib_type = libvk::InstanceType {
        dynamic_load: true,
        ..libvk::InstanceType::default()
    };

    let lib = libvk::Instance::new(&lib_type);

    assert!(lib.is_ok());
}