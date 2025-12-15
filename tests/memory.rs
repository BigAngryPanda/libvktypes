mod test_context;

#[cfg(test)]
mod memory {
    use libvktypes::{
        dev,
        extensions,
        hw,
        layers,
        libvk,
        memory
    };

    use super::test_context;

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

        let filter = |m: &hw::MemoryDescription, mask: u32| -> bool {
            let property = hw::MemoryProperty::HOST_VISIBLE;

            (mask >> m.index() & 1) == 1 && m.is_compatible(property)
        };

        let mem_cfg = [
            memory::LayoutElementCfg::Buffer {
                size: 1,
                usage: memory::STORAGE,
                queue_families: &[queue.index()],
                simultaneous_access: false,
                count: 1
            }
        ];

        assert!(memory::Memory::allocate(&device, &mut mem_cfg.iter(), &filter).is_ok());
    }

    fn multiple_buffers() {
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

        let memory_cfg = [
            memory::LayoutElementCfg::Buffer  {
                size: 42,
                usage: memory::STORAGE,
                queue_families: &[queue.index()],
                simultaneous_access: false,
                count: 2
            },
            memory::LayoutElementCfg::Buffer {
                size: 137,
                usage: memory::UNIFORM,
                queue_families: &[queue.index()],
                simultaneous_access: false,
                count: 1
            }
        ];

        assert!(memory::Memory::allocate_host_memory(&device, &mut memory_cfg.iter()).is_ok());
    }

    fn image_allocation() {
        let swp = test_context::get_swapchain();

        assert!(swp.images().is_ok());
    }

    fn depth_buffer() {
        let queue = test_context::get_graphics_queue();

        let caps = test_context::get_surface_capabilities();

        let alloc_info = [
            memory::LayoutElementCfg::Image {
                queue_families: &[queue.index()],
                simultaneous_access: false,
                format: memory::ImageFormat::D32_SFLOAT,
                extent: caps.extent3d(1),
                usage: memory::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                layout: memory::ImageLayout::UNDEFINED,
                aspect: memory::ImageAspect::DEPTH,
                tiling: memory::Tiling::OPTIMAL,
                count: 1
            }
        ];

        assert!(memory::Memory::allocate_device_memory(test_context::get_graphics_device(), &mut alloc_info.iter()).is_ok());
    }

    fn init_framebuffer() {
        let dev = test_context::get_graphics_device();

        let rp = test_context::get_render_pass();

        let images = test_context::get_image_list();

        let capabilities = test_context::get_surface_capabilities();

        let current_image = [memory::RefImageView::new(&images[0], 0)];

        let mut framebuffer_cfg = memory::FramebufferCfg {
            render_pass: rp,
            images: &mut current_image.iter(),
            extent: capabilities.extent2d()
        };

        assert!(memory::Framebuffer::new(dev, &mut framebuffer_cfg).is_ok());
    }

    fn access_buffers() {
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

        let memory_cfg = [
            memory::LayoutElementCfg::Buffer {
                size: 42,
                usage: memory::VERTEX,
                queue_families: &[queue.index()],
                simultaneous_access: false,
                count: 2
            },
            memory::LayoutElementCfg::Buffer {
                size: 137,
                usage: memory::UNIFORM,
                queue_families: &[queue.index()],
                simultaneous_access: false,
                count: 1
            }
        ];

        let memory = memory::Memory::allocate_host_memory(&device, &mut memory_cfg.iter()).expect("Failed to allocate memory");

        let result = memory.access(&mut |bytes: &mut [u8]| {
            bytes.clone_from_slice(&[0x42; 42]);
        }, 0);

        assert!(result.is_ok());

        let result = memory.access(&mut |bytes: &mut [u8]| {
            bytes.clone_from_slice(&[0xff; 137]);
        }, 2);

        assert!(result.is_ok());
    }

    fn multiple_images() {
        let queue = test_context::get_graphics_queue();

        let images_cfg = [
            memory::LayoutElementCfg::Image {
                queue_families: &[queue.index()],
                simultaneous_access: false,
                format: memory::ImageFormat::D32_SFLOAT,
                extent: memory::Extent3D {height: 800, width: 600, depth: 1 },
                usage: memory::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                layout: memory::ImageLayout::UNDEFINED,
                aspect: memory::ImageAspect::DEPTH,
                tiling: memory::Tiling::OPTIMAL,
                count: 1
            },
            memory::LayoutElementCfg::Image {
                queue_families: &[queue.index()],
                simultaneous_access: false,
                format: memory::ImageFormat::R8G8B8A8_SNORM,
                extent: memory::Extent3D {height: 1920, width: 1080, depth: 1 },
                usage: memory::ImageUsageFlags::COLOR_ATTACHMENT,
                layout: memory::ImageLayout::UNDEFINED,
                aspect: memory::ImageAspect::COLOR,
                tiling: memory::Tiling::OPTIMAL,
                count: 2
            }
        ];

        assert!(memory::Memory::allocate_device_memory(test_context::get_graphics_device(), &mut images_cfg.iter()).is_ok());
    }

    fn write_to_image() {
        let queue = test_context::get_graphics_queue();

        let capabilities = test_context::get_surface_capabilities();

        let images_cfg = [
            memory::LayoutElementCfg::Image {
                queue_families: &[queue.index()],
                simultaneous_access: false,
                format: capabilities.formats().next().expect("No available formats").format,
                extent: memory::Extent3D {height: 1920, width: 1080, depth: 1 },
                usage: memory::ImageUsageFlags::STORAGE | memory::ImageUsageFlags::TRANSFER_SRC | memory::ImageUsageFlags::TRANSFER_DST,
                layout: memory::ImageLayout::UNDEFINED,
                aspect: memory::ImageAspect::COLOR,
                tiling: memory::Tiling::LINEAR,
                count: 1
            }
        ];

        let memory = memory::Memory::allocate_host_memory(
            test_context::get_graphics_device(), &mut images_cfg.iter()
        ).expect("Failed to allocate image memory");

        let result = memory.access(&mut |bytes: &mut [u8]| {
            bytes.fill(0x42);
        }, 0);

        assert!(result.is_ok());
    }

    #[test]
    fn tests() {
        compute_memory_allocation();
        multiple_buffers();
        image_allocation();
        depth_buffer();
        init_framebuffer();
        access_buffers();
        multiple_images();
        write_to_image();
    }
}