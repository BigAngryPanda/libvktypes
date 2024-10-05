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
    pub(crate) fn new(storage: &memory::Memory, index: usize) -> View {
        View {
            i_memory: storage,
            i_index: index
        }
    }

    /// Return offset of the buffer
    pub fn offset(&self) -> u64 {
        self.i_memory.subregions()[self.i_index].offset
    }

    /// Return requested size of the buffer
    pub fn size(&self) -> u64 {
        self.i_memory.sizes()[self.i_index]
    }

    /// Return size of the buffer with respect to the alignment
    pub fn allocated_size(&self) -> u64 {
        self.i_memory.subregions()[self.i_index].allocated_size
    }

    /// Map selected region of memory
    ///
    /// Note: this is dangerous operation and you should use it with cautious
    /// As one range of the memory is mapped you *cannot* map another region of the same memory
    ///
    /// Better alternative is to [map full range](crate::memory::Memory::map_memory)
    /// and use [`mapped_slice`](Self::mapped_slice)
    pub fn map_memory<T>(&self) -> Result<&'a mut [T], memory::MemoryError> {
        self.i_memory.region().map_memory(self.offset(), self.size(), self.allocated_size())
    }

    /// Take the whole range and return part of it represented by the view
    ///
    /// View [size](Self::size) must be multiply of type size
    pub fn mapped_slice<T>(&self, mapped_memory: &mut [u8]) -> &'a mut [T] {
        debug_assert!(self.size() % (std::mem::size_of::<T>() as u64) == 0, "View size must be multiply of type size");

        unsafe {
            std::slice::from_raw_parts_mut(
                mapped_memory[self.offset() as usize..].as_mut_ptr() as *mut T,
                (self.size() as usize)/std::mem::size_of::<T>()) }
    }

    /// Execute `f` over selected buffer
    ///
    /// It is relatively expensive operation as memory will be mapped and unmapped
    ///
    /// It is better to use [`map_memory`](Self::map_memory) for frequent changes
    pub fn access<T, F>(&self, f: &mut F) -> Result<(), memory::MemoryError>
    where
        F: FnMut(&mut [T]),
    {
        self.i_memory.access(f, self.i_index)
    }

    /// Unmap memory by view
    ///
    /// Use for [`map_memory`](Self::map_memory)
    pub fn unmap_memory(&self) {
        self.i_memory.unmap_memory();
    }

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

    /// Return offset of the image buffer
    pub fn offset(&self) -> u64 {
        self.i_memory.subregions()[self.i_index].offset
    }

    /// Return size of the image buffer
    pub fn allocated_size(&self) -> u64 {
        self.i_memory.subregions()[self.i_index].allocated_size
    }

    /// Return image extent
    pub fn extent(&self) -> memory::Extent3D {
        self.i_memory.info()[self.i_index].extent
    }

    /// Map selected region of memory
    ///
    /// Note: this is dangerous operation and you should use it with cautious
    /// As one range of the memory is mapped you *cannot* map another region of the same memory
    ///
    /// Better alternative is to [map full range](crate::memory::Memory::map_memory)
    /// and use [`mapped_slice`](Self::mapped_slice)
    pub fn map_memory<T>(&self) -> Result<&mut [T], memory::MemoryError> {
        self.i_memory.region().map_memory(self.offset(), self.allocated_size(), self.allocated_size())
    }

    /// Take the whole range and return part of it represented by the view
    ///
    /// View [size](Self::allocated_size) must be multiply of type size
    pub fn mapped_slice<T>(&self, mapped_memory: &mut [u8]) -> &'a mut [T] {
        debug_assert!(self.allocated_size() % (std::mem::size_of::<T>() as u64) == 0, "View allocated size must be multiply of type size");

        unsafe {
            std::slice::from_raw_parts_mut(
                mapped_memory[self.offset() as usize..].as_mut_ptr() as *mut T,
                (self.allocated_size() as usize)/std::mem::size_of::<T>()) }
    }

    /// Execute 'f' over selected buffer
    pub fn access<T, F>(&self, f: &mut F) -> Result<(), memory::MemoryError>
    where
        F: FnMut(&mut [T]),
    {
        self.i_memory.access(f, self.i_index)
    }

    /// Return image aspect
    ///
    /// For swapchain images returns `ImageAspect::COLOR`
    pub fn aspect(&self) -> memory::ImageAspect {
        self.i_memory.info()[self.i_index].subresource.aspect_mask
    }

    /// Return image layout
    pub fn layout(&self) -> memory::ImageLayout {
        self.i_memory.info()[self.i_index].layout.get()
    }

    /// Unmap memory by view
    ///
    /// Use for [`map_memory`](Self::map_memory)
    pub fn unmap_memory(&self) {
        self.i_memory.unmap_memory();
    }

    pub(crate) fn set_layout(&self, new_layout: memory::ImageLayout) {
        self.i_memory.info()[self.i_index].layout.set(new_layout)
    }

    pub(crate) fn subresource_range(&self) -> vk::ImageSubresourceRange {
        self.i_memory.info()[self.i_index].subresource
    }

    pub(crate) fn subresource_layer(&self) -> vk::ImageSubresourceLayers {
        let subres = self.i_memory.info()[self.i_index].subresource;

        vk::ImageSubresourceLayers {
            aspect_mask: subres.aspect_mask,
            mip_level: subres.base_mip_level,
            base_array_layer: subres.base_array_layer,
            layer_count: subres.layer_count
        }
    }

    pub(crate) fn image_view(&self) -> vk::ImageView {
        self.i_memory.image_views()[self.i_index]
    }

    pub(crate) fn image(&self) -> vk::Image {
        self.i_memory.images()[self.i_index]
    }
}