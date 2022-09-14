use libvktypes::{
    dev,
    extensions,
    hw,
    layers,
    libvk,
    memory,
    shader,
    compute,
    cmd
};

#[path = "./mod.rs"]
mod test_context;

use std::ffi::CString;

#[test]
fn cmd_pool_allocation() {
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

    let cmd_pool_type = cmd::CmdPoolType {
        device: &device,
    };

    assert!(cmd::CmdPool::new(&cmd_pool_type).is_ok());
}

#[test]
fn cmd_buffer_exec() {
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
        size: 4,
        properties: hw::MemoryProperty::HOST_VISIBLE | hw::MemoryProperty::HOST_COHERENT | hw::MemoryProperty::HOST_CACHED,
        usage: memory::UsageFlags::STORAGE_BUFFER | memory::UsageFlags::TRANSFER_SRC | memory::UsageFlags::TRANSFER_DST,
        sharing_mode: memory::SharingMode::EXCLUSIVE,
        queue_families: &[device.queue_index()],
    };

    let buff = memory::Memory::allocate(&mem_type).expect("Failed to allocate memory");

    let shader_type = shader::ShaderType {
        device: &device,
        path: "tests/compiled_shaders/fill_memory.spv",
        entry: CString::new("main").expect("Failed to allocate string"),
    };

    let shader = shader::Shader::from_file(&shader_type).expect("Failed to create shader module");

    let pipe_type = compute::PipelineType {
        device: &device,
        buffers: &[&buff],
        shader: &shader,
        push_constant_size: 0,
    };

    let pipeline = compute::Pipeline::new(&pipe_type).expect("Failed to create pipeline");

    let cmd_pool_type = cmd::CmdPoolType {
        device: &device,
    };

    let cmd_pool = cmd::CmdPool::new(&cmd_pool_type).expect("Failed to allocate command pool");

    let mut cmd_buffer = cmd::CmdBuffer::default();

    cmd_buffer.bind_pipeline(&pipeline);

    cmd_buffer.dispatch(1, 1, 1);

    let queue_type = cmd::ComputeQueueType {
        cmd_pool: &cmd_pool,
        cmd_buffer: &cmd_buffer,
        queue_index: 0,
    };

    let cmd_queue = cmd::CompletedQueue::commit(&queue_type).expect("Failed to create command buffer");

    let exec_info = cmd::ExecInfo {
        wait_stage: cmd::PipelineStage::COMPUTE_SHADER,
        timeout: u64::MAX,
        wait: &[],
        signal: &[],
    };

    assert!(cmd_queue.exec(&exec_info).is_ok())
}

#[test]
fn write_graphics_cmds() {
    let render_pass = test_context::get_render_pass();

    let pipeline = test_context::get_graphics_pipeline();

    let framebuffer = test_context::get_framebuffers().framebuffers().next().expect("No available framebuffers");

    let pool = test_context::get_cmd_pool();

    let mut cmd_buffer = cmd::CmdBuffer::default();

    cmd_buffer.begin_render_pass(render_pass, framebuffer);

    cmd_buffer.bind_graphics_pipeline(pipeline);

    cmd_buffer.end_render_pass();

    let queue_type = cmd::ComputeQueueType {
        cmd_pool: pool,
        cmd_buffer: &cmd_buffer,
        queue_index: 0,
    };

    assert!(cmd::CompletedQueue::commit(&queue_type).is_ok());
}