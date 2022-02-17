#[cfg(test)]
mod tests {
    use libvktypes::instance::*;
    use libvktypes::hardware::*;
    use libvktypes::utils::filters::*;
    use libvktypes::logical_device::*;

    fn hw_selector(hw_desc: &HWDescription) -> bool {
        hw_desc.hw_type != HWType::CPU && hw_desc.hw_type != HWType::Unknown
    }

    #[test]
    fn default_instance_creation() {
        assert_eq!(LibHandler::with_default().is_ok(), true);
    }

    #[test]
    fn instance_creation() {
        assert_eq!(LibHandler::new(1, 0, 57, true).is_ok(), true);
    }

    #[test]
    fn hardware_inspection() {
        let instance = LibHandler::with_default().expect("Failed to create instance");
        let hw_list  = HWDescription::list(&instance);

        assert_eq!(hw_list.is_some(), true);

        // To enable stdout in tests run cargo test -- --nocapture
        // https://stackoverflow.com/questions/25106554/why-doesnt-println-work-in-rust-unit-tests
        for (i, hw) in hw_list.unwrap().iter().enumerate() {
            print!("\nDevice number {}\n", i);
            print!("{}", hw);
        }
    }

    #[test]
    fn logical_device_creation() {
        let instance = LibHandler::with_default().expect("Failed to create instance");
        let hw_list  = HWDescription::list(&instance).expect("No suitable devices");

        let hw_info = select_hw(hw_list.iter(), hw_selector, is_compute_family, any_memory);

        assert_eq!(hw_info.is_some(), true);

        let hw_dev_ref = &hw_list[hw_info.unwrap().device];

        let l_dev = LogicalDevice::new(&instance, hw_dev_ref, hw_info.unwrap().queue);

        assert_eq!(l_dev.is_some(), true);
    }
}