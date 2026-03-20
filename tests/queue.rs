mod test_context;

#[cfg(test)]
mod queue {
    use libvktypes::queue;

    use super::test_context;

    #[test]
    fn queue_alloc() {
        let graphics_queue = test_context::get_graphics_queue();

        let device = test_context::get_graphics_device();

        let _ = queue::Queue::new(device, graphics_queue.index(), 0);
    }
}