use libvktypes::{libvk, layers, extensions, hw, surface};

#[path = "./mod.rs"]
mod test_context;

#[cfg(target_os = "linux")]
#[test]
fn init_surface() {
    let window_ref = test_context::get_window();

    let lib_type = libvk::InstanceType {
        debug_layer: Some(layers::DebugLayer::default()),
        extensions: &[extensions::DEBUG_EXT_NAME,
            extensions::SURFACE_EXT_NAME,
            extensions::XLIB_SURFACE_EXT_NAME
        ],
        ..libvk::InstanceType::default()
    };

    let lib = libvk::Instance::new(&lib_type).expect("Failed to create instance");

    assert!(surface::Surface::new(&lib, window_ref).is_ok());
}

#[cfg(target_os = "linux")]
#[test]
fn get_capabilities() {
    let window_ref = test_context::get_window();

    let lib_type = libvk::InstanceType {
        debug_layer: Some(layers::DebugLayer::default()),
        extensions: &[extensions::DEBUG_EXT_NAME,
            extensions::SURFACE_EXT_NAME,
            extensions::XLIB_SURFACE_EXT_NAME
        ],
        ..libvk::InstanceType::default()
    };

    let lib = libvk::Instance::new(&lib_type).expect("Failed to create instance");

    let surface = surface::Surface::new(&lib, window_ref).expect("Failed to create surface");

    let hw_list = hw::Description::poll(&lib, Some(&surface)).expect("Failed to list hardware");

    let (hw_dev, _, _) = hw_list
        .find_first(
            hw::HWDevice::is_dedicated_gpu,
            |q| q.is_graphics() && q.is_surface_supported(),
            |_| true
        )
        .expect("Failed to find suitable hardware device");

    let cap_type = surface::CapabilitiesType {
        hw: hw_dev,
        surface: &surface
    };

    assert!(surface::Capabilities::get(&cap_type).is_ok());
}