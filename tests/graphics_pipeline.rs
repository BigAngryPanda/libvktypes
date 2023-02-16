#[path = "./mod.rs"]
mod test_context;

use libvktypes::{graphics, memory};

#[test]
fn create_pipeline() {
    let capabilities = test_context::get_surface_capabilities();

    let vertex_cfg = graphics::VertexInputCfg {
        location: 0,
        binding: 0,
        format: memory::ImageFormat::R32G32B32A32_SFLOAT,
        offset: 0,
    };

    let pipe_type = graphics::PipelineType {
        vertex_shader: test_context::get_vert_shader(),
        vertex_size: std::mem::size_of::<[f32; 2]>() as u32,
        vert_input: &[vertex_cfg],
        frag_shader: test_context::get_frag_shader(),
        topology: graphics::Topology::TRIANGLE_STRIP,
        extent: capabilities.extent2d(),
        push_constant_size: 0,
        render_pass: test_context::get_render_pass(),
        subpass_index: 0,
        enable_depth: false
    };

    assert!(graphics::Pipeline::new(test_context::get_graphics_device(), &pipe_type).is_ok());
}