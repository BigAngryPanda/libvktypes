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

        let compute_memory = memory::BufferCfg {
            size: 1,
            usage: memory::STORAGE,
            queue_families: &[queue.index()],
            simultaneous_access: false,
            count: 1
        };

        let mem_cfg = memory::MemoryCfg {
            properties: hw::MemoryProperty::HOST_VISIBLE,
            filter: &hw::any,
            buffers: &[&compute_memory]
        };

        assert!(memory::Memory::allocate(&device, &mem_cfg).is_ok());
    }

    #[test]
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

        let compute_memory = memory::BufferCfg {
            size: 42,
            usage: memory::STORAGE,
            queue_families: &[queue.index()],
            simultaneous_access: false,
            count: 2
        };

        let ubo = memory::BufferCfg {
            size: 137,
            usage: memory::UNIFORM,
            queue_families: &[queue.index()],
            simultaneous_access: false,
            count: 1
        };

        let mem_cfg = memory::MemoryCfg {
            properties: hw::MemoryProperty::HOST_VISIBLE,
            filter: &hw::any,
            buffers: &[&compute_memory, &ubo]
        };

        assert!(memory::Memory::allocate(&device, &mem_cfg).is_ok());
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

        let depth_buffer_cfg = [
            memory::ImageCfg {
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

        let alloc_info = memory::ImagesAllocationInfo {
            properties: hw::MemoryProperty::DEVICE_LOCAL,
            filter: &hw::any,
            image_cfgs: &depth_buffer_cfg
        };

        assert!(memory::ImageMemory::allocate(test_context::get_graphics_device(), &alloc_info).is_ok());
    }

    #[test]
    fn init_framebuffer() {
        let dev = test_context::get_graphics_device();

        let rp = test_context::get_render_pass();

        let images = test_context::get_image_list();

        let capabilities = test_context::get_surface_capabilities();

        let framebuffer_cfg = memory::FramebufferCfg {
            render_pass: rp,
            images: &[images[0].view(0)],
            extent: capabilities.extent2d()
        };

        assert!(memory::Framebuffer::new(dev, &framebuffer_cfg).is_ok());
    }

    #[test]
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

        let vertex_data = memory::BufferCfg {
            size: 42,
            usage: memory::VERTEX,
            queue_families: &[queue.index()],
            simultaneous_access: false,
            count: 1
        };

        let ubo = memory::BufferCfg {
            size: 137,
            usage: memory::UNIFORM,
            queue_families: &[queue.index()],
            simultaneous_access: false,
            count: 1
        };

        let mem_cfg = memory::MemoryCfg {
            properties: hw::MemoryProperty::HOST_VISIBLE,
            filter: &hw::any,
            buffers: &[&vertex_data, &ubo]
        };

        let memory = memory::Memory::allocate(&device, &mem_cfg).expect("Failed to allocate memory");

        let result = memory.access(&mut |bytes: &mut [u8]| {
            bytes.clone_from_slice(&[0x42; 42]);
        }, 0);

        assert!(result.is_ok());

        let result = memory.access(&mut |bytes: &mut [u8]| {
            bytes.clone_from_slice(&[0xff; 137]);
        }, 1);

        assert!(result.is_ok());
    }

    #[test]
    fn multiple_images() {
        let queue = test_context::get_graphics_queue();

        let images_cfg = [
            memory::ImageCfg {
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
            memory::ImageCfg {
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

        let alloc_info = memory::ImagesAllocationInfo {
            properties: hw::MemoryProperty::DEVICE_LOCAL,
            filter: &hw::any,
            image_cfgs: &images_cfg
        };

        assert!(memory::ImageMemory::allocate(test_context::get_graphics_device(), &alloc_info).is_ok());
    }

    #[test]
    fn write_to_image() {
        let queue = test_context::get_graphics_queue();

        let capabilities = test_context::get_surface_capabilities();

        let images_cfg = [
            memory::ImageCfg {
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

        let alloc_info = memory::ImagesAllocationInfo {
            properties: hw::MemoryProperty::HOST_VISIBLE,
            filter: &hw::any,
            image_cfgs: &images_cfg
        };

        let memory = memory::ImageMemory::allocate(test_context::get_graphics_device(), &alloc_info).expect("Failed to allocate image memory");

        let result = memory.view(0).access(&mut |bytes: &mut [u8]| {
            bytes.fill(0x42);
        });

        assert!(result.is_ok());
    }
}