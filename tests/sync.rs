mod test_context;

#[cfg(test)]
mod sync {
    use libvktypes::sync;

    use super::test_context;

    #[test]
    fn create_semaphore() {
        let dev = test_context::get_graphics_device();

        assert!(sync::Semaphore::new(dev).is_ok());
    }

    #[test]
    fn create_fence() {
        let dev = test_context::get_graphics_device();

        assert!(sync::Fence::new(dev, false).is_ok());

        assert!(sync::Fence::new(dev, true).is_ok());
    }
}