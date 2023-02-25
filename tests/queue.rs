mod test_context;

#[cfg(test)]
mod queue {
    use libvktypes::queue;

    use super::test_context;

    #[test]
    fn queue_alloc() {
        let graphics_queue = test_context::get_graphics_queue();

        let device = test_context::get_graphics_device();

        let cfg = queue::QueueCfg {
            family_index: graphics_queue.index(),
            queue_index: 0,
        };

        let _ = device.get_queue(&cfg);
    }
}