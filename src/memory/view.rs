//! Provide handler to the part of the [`Memory`](crate::memory::Memory)

use crate::memory;

use ash::vk;

/// "Pointer-like" struct for the buffer
#[derive(Debug, Clone, Copy)]
pub struct View<'a> {
    i_memory: &'a memory::Memory,
    i_index: usize
}

impl<'a> View<'a> {
    /// Create new view
    pub fn new(storage: &memory::Memory, index: usize) -> View {
        View {
            i_memory: storage,
            i_index: index
        }
    }

    /// Return offset of the buffer
    ///
    /// Same as [`buffer_offset`](crate::memory::Memory::buffer_offset)
    pub fn offset(&self) -> u64 {
        self.i_memory.buffer_offset(self.i_index)
    }

    /// Return size of the buffer
    ///
    /// Same as [`buffer_size`](crate::memory::Memory::buffer_size)
    pub fn size(&self) -> u64 {
        self.i_memory.buffer_size(self.i_index)
    }

    /// Return size of the buffer with respect to the alignment
    pub fn allocated_size(&self) -> u64 {
        self.i_memory.buffer_allocated_size(self.i_index)
    }

    /// Execute 'f' over selected buffer
    pub fn access<T, F>(&self, f: &mut F) -> Result<(), memory::MemoryError>
    where
        F: FnMut(&mut [T]),
    {
        self.i_memory.access(f, self.i_index)
    }

    #[doc(hidden)]
    pub(crate) fn buffer(&self) -> vk::Buffer {
        self.i_memory.buffer(self.i_index)
    }
}

/// "Pointer-like" struct for the buffer
#[derive(Debug, Clone, Copy)]
pub struct ImageView<'a> {
    i_memory: &'a memory::ImageMemory,
    i_index: usize
}

impl<'a> ImageView<'a> {
    pub(crate) fn new(storage: &memory::ImageMemory, index: usize) -> ImageView {
        ImageView {
            i_memory: storage,
            i_index: index
        }
    }

    /// Return offset of the buffer
    ///
    /// Same as [`buffer_offset`](crate::memory::Memory::buffer_offset)
    pub fn offset(&self) -> u64 {
        self.i_memory.subregions()[self.i_index].offset
    }

    /// Return size of the buffer
    ///
    /// Same as [`buffer_size`](crate::memory::Memory::buffer_size)
    pub fn size(&self) -> u64 {
        self.i_memory.subregions()[self.i_index].size
    }

    /// Return size of the buffer with respect to the alignment
    pub fn allocated_size(&self) -> u64 {
        self.i_memory.subregions()[self.i_index].allocated_size
    }

    /// REturn image extent
    pub fn extent(&self) -> memory::Extent3D {
        self.i_memory.extents()[self.i_index]
    }

    /// Execute 'f' over selected buffer
    pub fn access<T, F>(&self, f: &mut F) -> Result<(), memory::MemoryError>
    where
        F: FnMut(&mut [T]),
    {
        self.i_memory.access(f, self.i_index)
    }

    pub(crate) fn image_view(&self) -> vk::ImageView {
        self.i_memory.image_views()[self.i_index]
    }
}