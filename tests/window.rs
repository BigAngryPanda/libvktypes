use libvktypes::window;

#[test]
#[ignore]
fn init_window() {
    assert!(window::Window::new().is_ok());
}