#[path = "./mod.rs"]
mod test_context;

use libvktypes::{graphics, surface};

#[test]
fn create_pipeline() {
    let capabilities = test_context::get_surface_capabilities();

    let vertex_cfg = graphics::VertexInputCfg {
        location: 0,
        binding: 0,
        format: surface::ImageFormat::R32G32B32A32_SFLOAT,
        offset: 0,
    };

    let pipe_type = graphics::PipelineType {
        device: test_context::get_graphics_device(),
        vertex_shader: test_context::get_vert_shader(),
        vertex_size: std::mem::size_of::<[f32; 2]>() as u32,
        vert_slots: 1,
        vert_input: &[vertex_cfg],
        frag_shader: test_context::get_frag_shader(),
        topology: graphics::Topology::TRIANGLE_STRIP,
        extent: capabilities.extent2d(),
        push_constant_size: 0,
        render_pass: test_context::get_render_pass(),
        subpass_index: 0,
    };

    assert!(graphics::Pipeline::new(&pipe_type).is_ok());
}