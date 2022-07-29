use libvktypes::types::{
    lib,
    layers,
    extensions
};
use libvktypes::resources::lib::Instance;

#[test]
fn default_instance() {
    let lib = Instance::new(&lib::InstanceType::default());

    assert!(lib.is_ok());
}

#[test]
fn debug_instance() {
    let lib_type = lib::InstanceType {
        debug_layer: Some(layers::DebugLayer::default()),
        extensions: &[extensions::DEBUG_EXT_NAME],
        ..lib::InstanceType::default()
    };

    let lib = Instance::new(&lib_type);

    assert!(lib.is_ok());
}

#[test]
fn dynamic_load_instance() {
    let lib_type = lib::InstanceType {
        dynamic_load: true,
        ..lib::InstanceType::default()
    };

    let lib = Instance::new(&lib_type);

    assert!(lib.is_ok());
}