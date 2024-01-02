mod test_context;

#[cfg(test)]
mod surface {
    use libvktypes::{hw, surface};

    use super::test_context;

    #[test]
    fn init_surface() {
        let window_ref = test_context::get_window();

        let lib = test_context::get_graphics_instance();

        assert!(surface::Surface::new(&lib, window_ref).is_ok());
    }

    #[test]
    fn get_capabilities() {
        let window = test_context::get_window();

        let lib = test_context::get_graphics_instance();

        let surface = surface::Surface::new(&lib, window).expect("Failed to create surface");

        let hw_list = hw::Description::poll(&lib, Some(&surface)).expect("Failed to list hardware");

        let (hw_dev, _, _) = hw_list
            .find_first(
                hw::HWDevice::is_dedicated_gpu,
                |q| q.is_graphics() && q.is_surface_supported(),
                |_| true
            )
            .expect("Failed to find suitable hardware device");

        assert!(surface::Capabilities::get(&hw_dev, &surface).is_ok());
    }
}