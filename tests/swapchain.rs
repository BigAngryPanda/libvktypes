mod test_context;

#[cfg(test)]
mod swapchain {
    use libvktypes::{surface, swapchain, memory};

    use super::test_context;

    #[test]
    fn init_swapchain() {
        let lib_ref = test_context::get_graphics_instance();

        let surface_ref = test_context::get_surface();

        let device = test_context::get_graphics_device();

    /*
        We have to search for presentation queue to pass validation layer

        [Debug][Error][Validation]"vkCreateSwapchainKHR():
        pCreateInfo->surface is not known at this time to be supported for presentation by this device.
        The vkGetPhysicalDeviceSurfaceSupportKHR() must be called beforehand,
        and it must return VK_TRUE support with this surface for at least one queue family of this device.
        The Vulkan spec states:
        surface must be a surface that is supported by the device as determined using vkGetPhysicalDeviceSurfaceSupportKHR
        (https://www.khronos.org/registry/vulkan/specs/1.1-extensions/html/vkspec.html#VUID-VkSwapchainCreateInfoKHR-surface-01270)"
    */
        let _ = test_context::get_present_queue();

        let capabilities = test_context::get_surface_capabilities();

        assert!(capabilities.is_img_count_supported(3));
        assert!(capabilities.is_format_supported(surface::SurfaceFormat {
            format: memory::ImageFormat::B8G8R8A8_UNORM,
            color_space: memory::ColorSpace::SRGB_NONLINEAR,
        }));
        assert!(capabilities.is_mode_supported(swapchain::PresentMode::FIFO));
        assert!(capabilities.is_flags_supported(memory::UsageFlags::COLOR_ATTACHMENT));

        let swp_type = swapchain::SwapchainCfg {
            num_of_images: 3,
            format: memory::ImageFormat::B8G8R8A8_UNORM,
            color: memory::ColorSpace::SRGB_NONLINEAR,
            present_mode: swapchain::PresentMode::FIFO,
            flags: memory::UsageFlags::COLOR_ATTACHMENT,
            extent: capabilities.extent2d(),
            transform: capabilities.pre_transformation(),
            alpha: capabilities.alpha_composition(),
        };

        assert!(swapchain::Swapchain::new(lib_ref, device, surface_ref, &swp_type).is_ok());
    }
}