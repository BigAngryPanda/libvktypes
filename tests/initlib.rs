use libvktypes::{dev, extensions, hw, layers, libvk, surface, window};

#[cfg(target_os = "linux")]
pub fn get_graphics_instance() -> libvk::Instance {
    let lib_type = libvk::InstanceType {
        debug_layer: Some(layers::DebugLayer::default()),
        extensions: &[
            extensions::DEBUG_EXT_NAME,
            extensions::SURFACE_EXT_NAME,
            extensions::XLIB_SURFACE_EXT_NAME,
        ],
        ..libvk::InstanceType::default()
    };

    libvk::Instance::new(&lib_type).expect("Failed to create instance")
}

pub fn get_window() -> window::Window {
    window::Window::new().expect("Failed to create window")
}

pub fn get_surface(
    instance: &libvk::Instance,
    window_handler: &window::Window,
) -> surface::Surface {
    let surface_cfg = surface::SurfaceType {
        lib: instance,
        window: window_handler,
    };

    surface::Surface::new(&surface_cfg).expect("Failed to create surface")
}

pub fn get_surface_capabilities(
    hw_dev: &hw::HWDevice,
    srf: &surface::Surface,
) -> surface::Capabilities {
    let cap_type = surface::CapabilitiesType {
        hw: hw_dev,
        surface: srf,
    };

    surface::Capabilities::get(&cap_type).expect("Failed to query capabilities")
}

pub fn get_graphics_device<'a>(
    instance: &'a libvk::Instance,
    hw_dev: &'a hw::HWDevice,
    qf: &'a hw::QueueFamilyDescription,
) -> dev::Device<'a> {
    let dev_type = dev::DeviceType {
        lib: instance,
        hw: hw_dev,
        queue_family_index: qf.index(),
        priorities: &[1.0_f32],
        extensions: &[extensions::SWAPCHAIN_EXT_NAME],
    };

    dev::Device::new(&dev_type).expect("Failed to create device")
}
