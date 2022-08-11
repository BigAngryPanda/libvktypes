use libvktypes::window;

#[test]
fn init_window() {
    assert!(window::Window::new().is_ok());
}