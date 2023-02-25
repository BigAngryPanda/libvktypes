mod test_context;

#[cfg(test)]
mod memory {
    use libvktypes::{
        dev,
        extensions,
        hw,
        layers,
        libvk,
        memory,
        graphics
    };

    use super::test_context;

    #[test]
    fn compute_memory_allocation() {
        let lib_type = libvk::InstanceType {
            debug_layer: Some(layers::DebugLayer::default()),
            extensions: &[extensions::DEBUG_EXT_NAME],
            ..libvk::InstanceType::default()
        };

        let lib = libvk::Instance::new(&lib_type).expect("Failed to load library");
        let hw_list = hw::Description::poll(&lib, None).expect("Failed to list hardware");

        let (hw_dev, queue, _) = hw_list
            .find_first(
                hw::HWDevice::is_dedicated_gpu,
                hw::QueueFamilyDescription::is_compute,
                |_| true
            )
            .expect("Failed to find suitable hardware device");

        let dev_type = dev::DeviceCfg {
            lib: &lib,
            hw: hw_dev,
            extensions: &[],
            allocator: None,
        };

        let device = dev::Device::new(&dev_type).expect("Failed to create device");

        let mem_type = memory::StorageCfg {
            size: 4,
            properties: hw::MemoryProperty::HOST_VISIBLE,
            usage: memory::BufferUsageFlags::STORAGE_BUFFER |
                memory::BufferUsageFlags::TRANSFER_SRC   |
                memory::BufferUsageFlags::TRANSFER_DST,
            shared_access: false,
            queue_families: &[queue.index()],
        };

        let selected_memory = device.find_memory(hw::any, &mem_type).expect("No suitable memory");

        assert!(memory::Storage::allocate(&device, &selected_memory, &mem_type).is_ok());
    }

    #[test]
    fn zero_allocation() {
        let lib_type = libvk::InstanceType {
            debug_layer: Some(layers::DebugLayer::default()),
            extensions: &[extensions::DEBUG_EXT_NAME],
            ..libvk::InstanceType::default()
        };

        let lib = libvk::Instance::new(&lib_type).expect("Failed to load library");
        let hw_list = hw::Description::poll(&lib, None).expect("Failed to list hardware");

        let (hw_dev, queue, _) = hw_list
            .find_first(
                hw::HWDevice::is_dedicated_gpu,
                hw::QueueFamilyDescription::is_compute,
                |_| true
            )
            .expect("Failed to find suitable hardware device");

        let dev_type = dev::DeviceCfg {
            lib: &lib,
            hw: hw_dev,
            extensions: &[],
            allocator: None,
        };

        let device = dev::Device::new(&dev_type).expect("Failed to create device");

        let mem_type = memory::StorageCfg {
            size: 0,
            properties: hw::MemoryProperty::HOST_VISIBLE,
            usage: memory::BufferUsageFlags::STORAGE_BUFFER |
                memory::BufferUsageFlags::TRANSFER_SRC   |
                memory::BufferUsageFlags::TRANSFER_DST,
            shared_access: false,
            queue_families: &[queue.index()],
        };

        let selected_memory = device.find_memory(hw::any, &mem_type).expect("No suitable memory");

        assert!(memory::Storage::allocate(&device, &selected_memory, &mem_type).is_err());
    }

    #[test]
    fn image_allocation() {
        let swp = test_context::get_swapchain();

        assert!(swp.images().is_ok());
    }

    #[test]
    fn depth_buffer() {
        let queue = test_context::get_graphics_queue();

        let caps = test_context::get_surface_capabilities();

        let img_type = memory::ImageCfg {
            queue_families: &[queue.index()],
            format: memory::ImageFormat::D32_SFLOAT,
            extent: caps.extent3d(1),
            usage: memory::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            layout: graphics::ImageLayout::UNDEFINED,
            aspect: memory::ImageAspect::DEPTH,
            properties: hw::MemoryProperty::DEVICE_LOCAL,
        };

        assert!(memory::Image::new(test_context::get_graphics_device(), &img_type).is_ok());
    }

    #[test]
    fn init_framebuffer() {
        let dev = test_context::get_graphics_device();

        let rp = test_context::get_render_pass();

        let img = test_context::get_image_list();

        let capabilities = test_context::get_surface_capabilities();

        let framebuffer_cfg = memory::FramebufferCfg {
            render_pass: rp,
            images: &[&img[0]],
            extent: capabilities.extent2d()
        };

        assert!(memory::Framebuffer::new(dev, &framebuffer_cfg).is_ok());
    }
}