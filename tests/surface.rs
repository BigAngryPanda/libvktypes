use libvktypes::{window, libvk, layers, extensions, surface};

#[cfg(target_os = "linux")]
#[test]
fn init_surface() {
    let window = window::Window::new().expect("Failed to create window");

    let lib_type = libvk::InstanceType {
        debug_layer: Some(layers::DebugLayer::default()),
        extensions: &[extensions::DEBUG_EXT_NAME.as_ptr(),
            extensions::SURFACE_EXT_NAME.as_ptr(),
            extensions::XLIB_SURFACE_EXT_NAME.as_ptr()
        ],
        ..libvk::InstanceType::default()
    };

    let lib = libvk::Instance::new(&lib_type).expect("Failed to create instance");

    let surface_cfg = surface::SurfaceType {
        lib: &lib,
        window: &window,
    };

    assert!(surface::Surface::new(&surface_cfg).is_ok());
}