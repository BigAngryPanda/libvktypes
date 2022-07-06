use libvktypes::types::{
    lib_type,
    layers,
    extensions
};
use libvktypes::resources::lib_instance::LibInstance;

#[test]
fn default_instance() {
    let lib = LibInstance::new(&lib_type::LibInstanceType::default());

    assert!(lib.is_ok());
}

#[test]
fn debug_instance() {
    let lib_type = lib_type::LibInstanceType {
        debug_layer: Some(layers::DebugLayer::default()),
        extensions: &[extensions::DEBUG_EXT_NAME],
        ..lib_type::LibInstanceType::default()
    };

    let lib = LibInstance::new(&lib_type);

    assert!(lib.is_ok());
}

#[test]
fn dynamic_load_instance() {
    let lib_type = lib_type::LibInstanceType {
        dynamic_load: true,
        ..lib_type::LibInstanceType::default()
    };

    let lib = LibInstance::new(&lib_type);

    assert!(lib.is_ok());
}