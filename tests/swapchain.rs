use libvktypes::{surface, swapchain};

#[path = "./mod.rs"]
mod test_context;

#[cfg(target_os = "linux")]
#[test]
fn init_swapchain() {
    let lib_ref = test_context::get_graphics_instance();

    let surface_ref = test_context::get_surface();

    let device = test_context::get_graphics_device();

    let graphic_queue = test_context::get_graphics_queue();

    let present_queue = test_context::get_present_queue();

    assert_eq!(graphic_queue.index(), present_queue.index());

    let capabilities = test_context::get_surface_capabilities();

    assert!(capabilities.is_img_count_supported(3));
    assert!(capabilities.is_format_supported(surface::SurfaceFormat {
        format: surface::ImageFormat::B8G8R8A8_UNORM,
        color_space: surface::ColorSpace::SRGB_NONLINEAR,
    }));
    assert!(capabilities.is_mode_supported(surface::PresentMode::FIFO));
    assert!(capabilities.is_flags_supported(surface::UsageFlags::COLOR_ATTACHMENT));

    let swp_type = swapchain::SwapchainType {
        lib: lib_ref,
        dev: device,
        surface: surface_ref,
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
