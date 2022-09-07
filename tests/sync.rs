use libvktypes::sync;

#[path = "./mod.rs"]
mod test_context;

#[test]
fn create_semaphore() {
    let dev = test_context::get_graphics_device();

    let sem_cfg = sync::SemaphoreType {
        device: dev,
    };

    assert!(sync::Semaphore::new(&sem_cfg).is_ok());
}

#[test]
fn create_fence() {
    let dev = test_context::get_graphics_device();

    let fence_cfg = sync::FenceType {
        device: dev,
        signaled: false,
    };

    assert!(sync::Fence::new(&fence_cfg).is_ok());

    let fence_cfg = sync::FenceType {
        device: dev,
        signaled: true,
    };

    assert!(sync::Fence::new(&fence_cfg).is_ok());
}

