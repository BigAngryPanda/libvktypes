//! Provide handler to the part of the [`Memory`](crate::memory::Memory)

use ash::vk;

use crate::memory;

/// `BufferView` provides interface to the memory region
/// which represents [buffer](crate::memory::layout::BufferCfg)
///
/// Typically you need to implement only [`memory`](Self::memory) and [`index`](Self::index)
///
/// # Safety
///
/// Trying access memory region which was allocated as non-buffer will cause panic
pub trait BufferView : Copy + Clone {
    fn memory(&self) -> &memory::Memory;

    fn index(&self) -> usize;

    /// Return offset of the buffer
    fn offset(&self) -> u64 {
        self.memory().layout().offset(self.index())
    }

    /// Return requested size of the buffer
    fn size(&self) -> u64 {
        self.memory().layout().size(self.index())
    }

    /// Return size of the buffer with respect to the alignment
    fn allocated_size(&self) -> u64 {
        self.memory().layout().allocated_size(self.index())
    }

    /// Map selected region of memory
    ///
    /// Note: this is dangerous operation and you should use it with cautious
    /// As one range of the memory is mapped you *cannot* map another region of the same memory
    ///
    /// Better alternative is to [map full range](crate::memory::Memory::map_memory)
    /// and use [`mapped_slice`](Self::mapped_slice)
    fn map_memory<'a, 'b : 'a, T>(&'b self) -> Result<&'a mut [T], memory::MemoryError> {
        self.memory().region().map_memory(self.offset(), self.size(), self.allocated_size())
    }

    /// Execute `f` over selected buffer
    ///
    /// It is relatively expensive operation as memory will be mapped and unmapped
    ///
    /// It is better to use [`map_memory`](Self::map_memory) for frequent changes
    fn access<T, F>(&self, f: &mut F) -> Result<(), memory::MemoryError>
    where
        F: FnMut(&mut [T]),
    {
        self.memory().access(f, self.index())
    }

    /// Unmap memory by view
    ///
    /// Use for [`map_memory`](Self::map_memory)
    fn unmap_memory(&self) {
        self.memory().unmap_memory();
    }
}

/// `ImageView` provides interface to the memory region
/// which represents [image](crate::memory::layout::ImageCfg)
///
/// Typically you need to implement only [`memory`](Self::memory) and [`index`](Self::index)
///
/// # Safety
///
/// Trying access memory region which was allocated as non-image will cause panic
pub trait ImageView : Copy + Clone {
    fn memory(&self) -> &memory::Memory;

    fn index(&self) -> usize;

    /// Return offset of the buffer
    fn offset(&self) -> u64 {
        self.memory().layout().offset(self.index())
    }

    /// Return size of the buffer with respect to the alignment
    fn allocated_size(&self) -> u64 {
        self.memory().layout().allocated_size(self.index())
    }

    /// Map selected region of memory
    ///
    /// Note: this is dangerous operation and you should use it with cautious
    /// As one range of the memory is mapped you *cannot* map another region of the same memory
    ///
    /// Better alternative is to [map full range](crate::memory::Memory::map_memory)
    /// and use [`mapped_slice`](Self::mapped_slice)
    fn map_memory<'a, 'b : 'a, T>(&'b self) -> Result<&'a mut [T], memory::MemoryError> {
        self.memory().region().map_memory(self.offset(), self.allocated_size(), self.allocated_size())
    }

    /// Execute `f` over selected buffer
    ///
    /// It is relatively expensive operation as memory will be mapped and unmapped
    ///
    /// It is better to use [`map_memory`](Self::map_memory) for frequent changes
    fn access<T, F>(&self, f: &mut F) -> Result<(), memory::MemoryError>
    where
        F: FnMut(&mut [T]),
    {
        self.memory().access(f, self.index())
    }

    /// Unmap memory by view
    ///
    /// Use for [`map_memory`](Self::map_memory)
    fn unmap_memory(&self) {
        self.memory().unmap_memory();
    }

    /// Return image extent
    fn extent(&self) -> memory::Extent3D {
        self.memory().layout().extent(self.index())
    }

    /// Return image aspect
    ///
    /// For swapchain images returns `ImageAspect::COLOR`
    fn aspect(&self) -> memory::ImageAspect {
        self.memory().layout().subresource(self.index()).aspect_mask
    }

    /// Return image format
    fn format(&self) -> memory::ImageFormat {
        self.memory().layout().format(self.index())
    }
}

/// "Pointer-like" struct for the buffer
#[derive(Debug, Clone, Copy)]
pub struct RefView<'a> {
    i_memory: &'a memory::Memory,
    i_index: usize
}

impl<'a> RefView<'a> {
    pub fn new(storage: &'_ memory::Memory, index: usize) -> RefView<'_> {
        RefView {
            i_memory: storage,
            i_index: index
        }
    }
}

impl<'a> BufferView for RefView<'a> {
    fn memory(&self) -> &memory::Memory {
        self.i_memory
    }

    fn index(&self) -> usize {
        self.i_index
    }
}

/// "Pointer-like" struct for the image
///
/// Mapping image memory is tight to a image [`layout`](Self::memory::ImageLayout)
#[derive(Debug, Clone, Copy)]
pub struct RefImageView<'a> {
    i_memory: &'a memory::Memory,
    i_index: usize
}

impl<'a> RefImageView<'a> {
    pub fn new(storage: &'_ memory::Memory, index: usize) -> RefImageView<'_> {
        RefImageView {
            i_memory: storage,
            i_index: index
        }
    }
}

impl<'a> ImageView for RefImageView<'a> {
    fn memory(&self) -> &memory::Memory {
        self.i_memory
    }

    fn index(&self) -> usize {
        self.i_index
    }
}

pub(crate) fn get_buffer<T: BufferView>(view: T) -> vk::Buffer {
    view.memory().layout().buffer(view.index())
}

pub(crate) fn get_image_view<T: ImageView>(view: T) -> vk::ImageView {
    view.memory().layout().image_view(view.index())
}

pub(crate) fn get_image<T: ImageView>(view: T) -> vk::Image {
    view.memory().layout().image(view.index())
}

pub(crate) fn get_subresource<T: ImageView>(view: T) -> vk::ImageSubresourceRange {
    view.memory().layout().subresource(view.index())
}
