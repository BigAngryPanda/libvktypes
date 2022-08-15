use libvktypes::{hw, surface, swapchain};

#[path = "./initlib.rs"]
mod initlib;

#[cfg(target_os = "linux")]
#[test]
#[ignore]
fn init_swapchain() {
    let window = initlib::get_window();

    let lib = initlib::get_graphics_instance();

    let surface = initlib::get_surface(&lib, &window);

    let hw_list = hw::Description::poll(&lib).expect("Failed to list hardware");

    let (hw_dev, qf, _) = hw_list
        .find_first(
            hw::HWDevice::is_discrete_gpu,
            hw::QueueFamilyDescription::is_graphics,
            |_| true,
        )
        .expect("Failed to find suitable hardware device");

    assert!(qf.support_surface(&surface, hw_dev));

    let device = initlib::get_graphics_device(&lib, hw_dev, qf);

    let capabilities = initlib::get_surface_capabilities(hw_dev, &surface);

    assert!(capabilities.is_img_count_supported(3));
    assert!(capabilities.is_format_supported(surface::SurfaceFormat {
        format: surface::ImageFormat::B8G8R8A8_UNORM,
        color_space: surface::ColorSpace::SRGB_NONLINEAR,
    }));
    assert!(capabilities.is_mode_supported(surface::PresentMode::FIFO));
    assert!(capabilities.is_flags_supported(surface::UsageFlags::COLOR_ATTACHMENT));

    let swp_type = swapchain::SwapchainType {
        lib: &lib,
        dev: &device,
        surface: &surface,
        num_of_images: 3,
        format: surface::ImageFormat::B8G8R8A8_UNORM,
        color: surface::ColorSpace::SRGB_NONLINEAR,
        present_mode: surface::PresentMode::FIFO,
        flags: surface::UsageFlags::COLOR_ATTACHMENT,
        extent: capabilities.extent2d(),
        transform: capabilities.pre_transformation(),
        alpha: capabilities.alpha_composition(),
    };

    assert!(swapchain::Swapchain::new(&swp_type).is_ok());
}
