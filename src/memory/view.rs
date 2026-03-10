//! Provide handler to the part of the [`Memory`](crate::memory::Memory)

use ash::vk;

use crate::memory;

/// `BufferView` provides interface to the memory region
/// which represents [buffer](crate::memory::layout::LayoutElementCfg)
///
/// Typically you need to implement only [`memory`](Self::memory) and [`index`](Self::index)
///
/// # Safety
///
/// Trying access memory region which was allocated as non-buffer will cause panic
pub trait BufferView : Copy + Clone {
    fn memory(&self) -> &memory::Memory;

    fn index(&self) -> usize;

    /// Return offset of the buffer in bytes
    fn offset(&self) -> u64 {
        self.memory().layout().offset(self.index())
    }

    /// Return requested size of the buffer in bytes
    fn size(&self) -> u64 {
        self.memory().layout().size(self.index())
    }

    /// Return size of the buffer with respect to the alignment in bytes
    fn allocated_size(&self) -> u64 {
        self.memory().layout().allocated_size(self.index())
    }

    /// Map selected region of memory
    ///
    /// Note: this is dangerous operation and you should use it with cautious
    /// As one range of the memory is mapped you *cannot* map another region of the same memory
    ///
    /// You must unmap memory with [`unmap_memory`](Self::unmap_memory)
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

    /// Return [`BufferView`](BufferView) part of the allocated memory as slice
    ///
    /// 1. It tries to find start and end indices by calling [`offset`](Self::offset)
    /// and [`size`](Self::size)
    ///
    /// 2. View calls [`align_to_mut`](slice::align_to_mut) to get properly aligned slice from step 1
    fn subslice<'a, T, U>(&self, data: &'a mut [T]) -> &'a mut [U] {
        let first_elem = (self.offset() / std::mem::size_of::<T>() as u64) as usize;
        let last_elem = first_elem + (self.size() / std::mem::size_of::<T>() as u64) as usize;

        unsafe { data[first_elem..last_elem].align_to_mut::<U>().1 }
    }
}

/// `ImageView` provides interface to the memory region
/// which represents [image](crate::memory::layout::LayoutElementCfg)
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
/// Mapping image memory is tight to a image [`layout`](memory::layout::LayoutElementCfg::Image)
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
