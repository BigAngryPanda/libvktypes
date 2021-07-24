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
        let hw_list = HWDevice::list(&instance);

        assert_eq!(hw_list.is_some(), true);
    }
}