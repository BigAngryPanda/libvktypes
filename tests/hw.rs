mod test_context;

#[cfg(test)]
mod hw {
    use libvktypes::{
        libvk,
        hw,
        layers,
        extensions
    };

    use super::test_context;

    #[test]
    fn hardware_inspection() {
        let lib_type = libvk::InstanceType {
            debug_layer: Some(layers::DebugLayer::default()),
            extensions: &[extensions::DEBUG_EXT_NAME],
            ..libvk::InstanceType::default()
        };

        let lib = libvk::Instance::new(&lib_type).expect("Failed to load library");
        let hw_list = hw::Description::poll(&lib, None).expect("Failed to list hardware");

        // To enable stdout in tests run cargo test -- --nocapture
        // https://stackoverflow.com/questions/25106554/why-doesnt-println-work-in-rust-unit-tests
        for (i, hw) in hw_list.list().enumerate() {
            print!("\nDevice number {}\n", i);
            print!("{}", hw);
        }
    }

    #[test]
    fn offset_calculation() {
        let hw_dev = test_context::get_graphics_hw();

        assert!(hw_dev.ubo_size(0) == 0);
        assert!(hw_dev.ubo_size(hw_dev.ubo_offset()) == hw_dev.ubo_offset());
        assert!(hw_dev.ubo_size(12345) % hw_dev.ubo_offset() == 0);
    }
}