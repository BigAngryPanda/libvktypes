use libvktypes::types::{
    lib,
    layers,
    extensions
};
use libvktypes::resources::{
    lib::Instance,
    hw::HWDescription,
};

#[test]
fn hardware_inspection() {
    let lib_type = lib::InstanceType {
        debug_layer: Some(layers::DebugLayer::default()),
        extensions: &[extensions::DEBUG_EXT_NAME],
        ..lib::InstanceType::default()
    };

    let lib = Instance::new(&lib_type).expect("Failed to load library");
    let hw_list = HWDescription::new(&lib).expect("Failed to list hardware");

    // To enable stdout in tests run cargo test -- --nocapture
    // https://stackoverflow.com/questions/25106554/why-doesnt-println-work-in-rust-unit-tests
    for (i, hw) in hw_list.list().enumerate() {
        print!("\nDevice number {}\n", i);
        print!("{}", hw);
    }
}