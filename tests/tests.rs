pub mod ut;

#[cfg(test)]
mod tests {
    use libvktypes::instance::*;
    use libvktypes::hardware::*;
    use libvktypes::utils::filters::*;
    use libvktypes::logical_device::*;
    use libvktypes::memory::*;

    #[test]
    fn default_instance_creation() {
        assert!(LibHandler::with_default().is_ok());
    }

    #[test]
    fn instance_creation() {
        assert!(LibHandler::new(1, 0, 57, true).is_ok());
    }

    #[test]
    fn hardware_inspection() {
        let instance = LibHandler::with_default().expect("Failed to create instance");
        let hw_list  = HWDescription::list(&instance);

        assert!(hw_list.is_some());

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

        let hw_info = select_hw(hw_list.iter(), dedicated_hw, is_compute_family, any_memory)
                        .expect("Failed to get device information");

        let hw_dev_ref = &hw_list[hw_info.device];

        let l_dev = LogicalDevice::new(&instance, hw_dev_ref, hw_info.queue);

        assert!(l_dev.is_some());
    }

    #[test]
    fn memory_allocation() {
        let instance = LibHandler::with_default().expect("Failed to create instance");
        let hw_list  = HWDescription::list(&instance).expect("No suitable devices");

        let hw_info = select_hw(hw_list.iter(), dedicated_hw, is_compute_family, any_memory)
                        .expect("Failed to get device information");

        let hw_dev_ref = &hw_list[hw_info.device];

        let l_dev = LogicalDevice::new(&instance, hw_dev_ref, hw_info.queue).expect("Failed to create logical device");

        let test_memory = Memory::new(&l_dev, 1,
            MemoryProperty::HOST_VISIBLE,
            BufferType::STORAGE_BUFFER | BufferType::TRANSFER_SRC | BufferType::TRANSFER_DST);

        assert!(test_memory.is_ok());

        let fail_test_memory = Memory::new(&l_dev, 0,
            MemoryProperty::HOST_VISIBLE,
            BufferType::STORAGE_BUFFER | BufferType::TRANSFER_SRC | BufferType::TRANSFER_DST);

        assert!(fail_test_memory.is_err());
    }
}