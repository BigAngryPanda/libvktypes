mod test_context;

#[cfg(test)]
mod render_pass {
    use libvktypes::{graphics, memory};

    use super::test_context;

    #[test]
    fn render_pass_init() {
        let dev = test_context::get_graphics_device();

        let cfg = test_context::get_surface_capabilities();

        let subpass_sync = [
            graphics::SubpassSync {
                src_subpass: graphics::SUBPASS_EXTERNAL,
                dst_subpass: 0,
                src_stage: graphics::PipelineStage::BOTTOM_OF_PIPE,
                dst_stage: graphics::PipelineStage::COLOR_ATTACHMENT_OUTPUT,
                src_access: graphics::AccessFlags::MEMORY_READ,
                dst_access: graphics::AccessFlags::COLOR_ATTACHMENT_WRITE,
            },
            graphics::SubpassSync {
                src_subpass: 0,
                dst_subpass: graphics::SUBPASS_EXTERNAL,
                src_stage: graphics::PipelineStage::COLOR_ATTACHMENT_OUTPUT,
                dst_stage: graphics::PipelineStage::BOTTOM_OF_PIPE,
                src_access: graphics::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_access: graphics::AccessFlags::MEMORY_READ,
            }
        ];

        let attachment = [
            graphics::AttachmentInfo {
                format: cfg.formats().next().expect("No available formats").format,
                load_op: graphics::AttachmentLoadOp::CLEAR,
                store_op: graphics::AttachmentStoreOp::STORE,
                stencil_load_op: graphics::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: graphics::AttachmentStoreOp::DONT_CARE,
                initial_layout: memory::ImageLayout::PRESENT_SRC_KHR,
                final_layout: memory::ImageLayout::PRESENT_SRC_KHR,
            }
        ];

        let subpass_info = [
            graphics::SubpassInfo {
                color_attachments: &[0],
                ..Default::default()
            }
        ];

        let rp_cfg = graphics::RenderPassCfg {
            attachments: &attachment,
            sync_info: &subpass_sync,
            subpasses: &subpass_info,
        };

        assert!(graphics::RenderPass::new(dev, &rp_cfg).is_ok());
    }
}