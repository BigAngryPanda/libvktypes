use libvktypes::{
    dev,
    extensions,
    hw,
    layers,
    libvk,
    memory,
};

#[test]
fn compute_memory_allocation() {
    let lib_type = libvk::InstanceType {
        debug_layer: Some(layers::DebugLayer::default()),
        extensions: &[extensions::DEBUG_EXT_NAME],
        ..libvk::InstanceType::default()
    };

    let lib = libvk::Instance::new(&lib_type).expect("Failed to load library");
    let hw_list = hw::Description::poll(&lib).expect("Failed to list hardware");

    let (hw_dev, queue, _) = hw_list
        .find_first(
            //|dev| hw::HWDevice::is_discrete_gpu(dev) || hw::HWDevice::is_integrated_gpu(dev),
            hw::HWDevice::is_discrete_gpu,
            hw::QueueFamilyDescription::is_compute,
            |_| true,
        )
        .expect("Failed to find suitable hardware device");

    let dev_type = dev::DeviceType {
        lib: &lib,
        hw: hw_dev,
        queue_family_index: queue.index(),
        priorities: &[1.0_f32],
        extensions: &[],
    };

    let device = dev::Device::new(&dev_type).expect("Failed to create device");

    let mem_type = memory::MemoryType {
        device: &device,
        size: 1,
        properties: hw::MemoryProperty::HOST_VISIBLE,
        usage: memory::UsageFlags::STORAGE_BUFFER | memory::UsageFlags::TRANSFER_SRC | memory::UsageFlags::TRANSFER_DST,
        sharing_mode: memory::SharingMode::EXCLUSIVE,
        queue_families: &[device.queue_index()],
    };

    assert!(memory::Memory::allocate(&mem_type).is_ok());
}

#[test]
fn zero_allocation() {
    let lib_type = libvk::InstanceType {
        debug_layer: Some(layers::DebugLayer::default()),
        extensions: &[extensions::DEBUG_EXT_NAME],
        ..libvk::InstanceType::default()
    };

    let lib = libvk::Instance::new(&lib_type).expect("Failed to load library");
    let hw_list = hw::Description::poll(&lib).expect("Failed to list hardware");

    let (hw_dev, queue, _) = hw_list
        .find_first(
            //|dev| hw::HWDevice::is_discrete_gpu(dev) || hw::HWDevice::is_integrated_gpu(dev),
            hw::HWDevice::is_discrete_gpu,
            hw::QueueFamilyDescription::is_compute,
            |_| true,
        )
        .expect("Failed to find suitable hardware device");

    let dev_type = dev::DeviceType {
        lib: &lib,
        hw: hw_dev,
        queue_family_index: queue.index(),
        priorities: &[1.0_f32],
        extensions: &[],
    };

    let device = dev::Device::new(&dev_type).expect("Failed to create device");

    let mem_type = memory::MemoryType {
        device: &device,
        size: 0,
        properties: hw::MemoryProperty::HOST_VISIBLE,
        usage: memory::UsageFlags::STORAGE_BUFFER | memory::UsageFlags::TRANSFER_SRC | memory::UsageFlags::TRANSFER_DST,
        sharing_mode: memory::SharingMode::EXCLUSIVE,
        queue_families: &[device.queue_index()],
    };

    assert!(memory::Memory::allocate(&mem_type).is_err());
}