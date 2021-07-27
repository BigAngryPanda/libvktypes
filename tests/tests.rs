#[cfg(test)]
mod tests {
    use libvktypes::instance::*;
    use libvktypes::hardware::*;

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
        let hw_list = HWDescription::list(&instance);

        assert_eq!(hw_list.is_some(), true);

        // To enable stdout in tests run cargo test -- --nocapture
        // https://stackoverflow.com/questions/25106554/why-doesnt-println-work-in-rust-unit-tests
        for (i, hw) in hw_list.unwrap().iter().enumerate() {
            print!("\nDevice number {}\n", i);
            print!("{}", hw);
        }
    }
}