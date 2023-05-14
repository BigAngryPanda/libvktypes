//! Information how to properly bind vertex buffer to the layout

use ash::vk;

use crate::{
    memory,
    graphics
};

/// `VertexView` contains information about
/// how vertex memory should be [bounded](crate::cmd::Buffer::bind_vertex_buffers)
/// to the layout in the vertex shader
///
/// Note: you may bind single vertex buffer to the different layouts (with different offsets)
///
/// See docs for [`VertexInputCfg`](graphics::VertexInputCfg)
#[derive(Debug, Clone)]
pub struct VertexView<'a> {
    i_view: memory::View<'a>,
    i_offset: u32
}

impl<'a> VertexView<'a> {
    pub fn from_cfg(view: memory::View<'a>, cfg: graphics::VertexInputCfg) -> VertexView<'a> {
        VertexView {
            i_view: view,
            i_offset: cfg.offset
        }
    }

    /// About `offset` read docs for [`VertexInputCfg`](graphics::VertexInputCfg)
    pub fn with_offset(view: memory::View<'a>, offset: u32) -> VertexView<'a> {
        VertexView {
            i_view: view,
            i_offset: offset
        }
    }

    pub fn offset(&self) -> u32 {
        self.i_offset
    }

    pub(crate) fn buffer(&self) -> vk::Buffer {
        self.i_view.buffer()
    }
}