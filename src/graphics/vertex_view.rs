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
pub struct VertexView<T : memory::BufferView> {
    i_view: T,
    i_offset: u32
}

impl<T : memory::BufferView> VertexView<T> {
    pub fn new(view: T) -> VertexView<T> {
        VertexView {
            i_view: view,
            i_offset: 0
        }
    }

    pub fn from_cfg(view: T, cfg: graphics::VertexInputCfg) -> VertexView<T> {
        VertexView {
            i_view: view,
            i_offset: cfg.offset
        }
    }

    /// About `offset` read docs for [`VertexInputCfg`](graphics::VertexInputCfg)
    pub fn with_offset(view: T, offset: u32) -> VertexView<T> {
        VertexView {
            i_view: view,
            i_offset: offset
        }
    }

    pub fn offset(&self) -> u32 {
        self.i_offset
    }

    pub(crate) fn buffer(&self) -> vk::Buffer {
        memory::get_buffer(self.i_view)
    }
}
