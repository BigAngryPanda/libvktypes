use libvktypes::{window, libvk, layers, extensions, hw, surface};

#[cfg(target_os = "linux")]
#[test]
fn init_surface() {
    let window = window::Window::new().expect("Failed to create window");

    let lib_type = libvk::InstanceType {
        debug_layer: Some(layers::DebugLayer::default()),
        extensions: &[extensions::DEBUG_EXT_NAME,
            extensions::SURFACE_EXT_NAME,
            extensions::XLIB_SURFACE_EXT_NAME
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

#[cfg(target_os = "linux")]
#[test]
fn get_capabilities() {
    let window = window::Window::new().expect("Failed to create window");

    let lib_type = libvk::InstanceType {
        debug_layer: Some(layers::DebugLayer::default()),
        extensions: &[extensions::DEBUG_EXT_NAME,
            extensions::SURFACE_EXT_NAME,
            extensions::XLIB_SURFACE_EXT_NAME
        ],
        ..libvk::InstanceType::default()
    };

    let lib = libvk::Instance::new(&lib_type).expect("Failed to create instance");

    let hw_list = hw::Description::poll(&lib).expect("Failed to list hardware");

    let (hw_dev, _, _) = hw_list
        .find_first(
            hw::HWDevice::is_discrete_gpu,
            hw::QueueFamilyDescription::is_graphics,
            |_| true,
        )
        .expect("Failed to find suitable hardware device");

    let surface_cfg = surface::SurfaceType {
        lib: &lib,
        window: &window,
    };

    let surface = surface::Surface::new(&surface_cfg).expect("Failed to create surface");

    let cap_type = surface::CapabilitiesType {
        hw: hw_dev,
        surface: &surface
    };

    assert!(surface::Capabilities::get(&cap_type).is_ok());
}