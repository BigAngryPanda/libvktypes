//! Represents memory for various purposes such as vertex buffer, uniform buffer etc.
use ash::vk;

use crate::{
    dev,
    hw,
    memory,
    on_option_ret,
    on_error_ret
};

use memory::layout::{
    LayoutElementCfg,
    Extent3D,
    ImageAspect
};

use memory::{
    LayoutElement,
    Layout,
    BufferUsageFlags
};

use std::sync::Arc;
use std::ptr;
use std::fmt;
use std::marker::PhantomData;

// Workaround
pub const FULL_TRANSFER: BufferUsageFlags = BufferUsageFlags::from_raw(
    (BufferUsageFlags::TRANSFER_SRC).as_raw() | (BufferUsageFlags::TRANSFER_DST).as_raw()
);

pub const STORAGE: BufferUsageFlags = BufferUsageFlags::from_raw(
    FULL_TRANSFER.as_raw() | (BufferUsageFlags::STORAGE_BUFFER).as_raw()
);

pub const UNIFORM: BufferUsageFlags = BufferUsageFlags::from_raw(
    FULL_TRANSFER.as_raw() | (BufferUsageFlags::UNIFORM_BUFFER).as_raw()
);

pub const VERTEX: BufferUsageFlags = BufferUsageFlags::from_raw(
    FULL_TRANSFER.as_raw() | (BufferUsageFlags::VERTEX_BUFFER).as_raw()
);

pub const INDEX: BufferUsageFlags = BufferUsageFlags::from_raw(
    FULL_TRANSFER.as_raw() | (BufferUsageFlags::INDEX_BUFFER).as_raw()
);

/// Size of the indices
///
#[doc = "Ash documentation about possible values <https://docs.rs/ash/latest/ash/vk/struct.IndexType.html>"]
///
#[doc = "Vulkan documentation <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkIndexType.html>"]
pub type IndexBufferType = vk::IndexType;

/// Special value for starting reassembly
pub const INDEX_REASSEMBLY_UINT32: u32 = 0xffffffff;
/// Special value for starting reassembly
pub const INDEX_REASSEMBLY_UINT16: u16 = 0xffff;
/// Special value for starting reassembly
pub const INDEX_REASSEMBLY_UINT8: u8 = 0xff;

/// Aligned region of memory
///
/// # Allocation
///
/// Memory allocated in single chunk
/// in order which is provided by [`LayoutElementCfg`](crate::memory::layout::LayoutElementCfg)
/// so no rearranges will be performed
///
/// Size of allocated memory is greater or equal to the requested size
/// (sum of all [`BufferCfg::size`] in [`MemoryCfg::buffers`]) due to alignment requirements
///
/// Functions like `allocate_host_memory` use default memory filter
///
/// Default filter checks only memory_bits from buffers and memory type flags
///
/// See also (`allocate`)[Self::allocate]
///
/// # Alignment
///
/// Each buffer from [`MemoryCfg::buffers`] will be separately aligned at least
/// for [`hw::memory_alignment`](crate::hw::HWDevice::memory_alignment)
///
/// Various types of buffers (uniform, storage etc.) may have their own alignment requirements
/// such as [`hw::ubo_offset`](crate::hw::HWDevice::ubo_offset) or [`hw::storage_offset`](crate::hw::HWDevice::storage_offset)
///
/// Hint: you may print struct (as [`Memory`] implements [`fmt::Display`]) to see memory layout
///
/// # Memory View
///
/// Whole memory chunk is split into regions (buffers) which are defined by [`MemoryCfg::buffers`]
///
/// To help with managing regions [`Memory View`](crate::memory::View) struct was provided
pub struct Memory {
    i_core: Arc<dev::Core>,
    i_layout: memory::layout::Layout,
    i_memory: memory::Region
}

impl Memory {
    /// Allocate memory with (`hw::MemoryProperty::HOST_VISIBLE`)[hw::MemoryProperty::HOST_VISIBLE] flag
    pub fn allocate_host_memory(
        device: &dev::Device,
        cfgs: &mut dyn Iterator<Item = &LayoutElementCfg>
    ) -> Result<Memory, memory::MemoryError> {
        Self::with_property(device, cfgs, hw::MemoryProperty::HOST_VISIBLE)
    }

    /// Allocate memory with (`
    /// hw::MemoryProperty::HOST_VISIBLE | hw::MemoryProperty::HOST_COHERENT | hw::MemoryProperty::HOST_CACHED`)
    /// [hw::MemoryProperty::HOST_VISIBLE] flag
    pub fn allocate_host_coherent_memory(
        device: &dev::Device,
        cfgs: &mut dyn Iterator<Item = &LayoutElementCfg>
    ) -> Result<Memory, memory::MemoryError> {
        Self::with_property(
            device,
            cfgs,
            hw::MemoryProperty::HOST_VISIBLE  |
            hw::MemoryProperty::HOST_COHERENT |
            hw::MemoryProperty::HOST_CACHED)
    }

    /// Allocate memory with (`hw::MemoryProperty::DEVICE_LOCAL`)[hw::MemoryProperty::DEVICE_LOCAL] flag
    pub fn allocate_device_memory(
        device: &dev::Device,
        cfgs: &mut dyn Iterator<Item = &LayoutElementCfg>
    ) -> Result<Memory, memory::MemoryError> {
        Self::with_property(device, cfgs, hw::MemoryProperty::DEVICE_LOCAL)
    }

    /// Allocate memory with selected [`property`](hw::MemoryProperty)
    pub fn with_property(
        device: &dev::Device,
        cfgs: &mut dyn Iterator<Item = &LayoutElementCfg>,
        property: hw::MemoryProperty,
    ) -> Result<Memory, memory::MemoryError> {
        let memory_filter = |m: &hw::MemoryDescription, bitmask: u32| -> bool {
            (bitmask >> m.index() & 1) == 1 &&
            m.is_compatible(property)
        };

        Self::allocate(device, cfgs, &memory_filter)
    }

    /// Allocate memory with custom memory filter
    ///
    /// Purpose of the filter is to give you full freedom in what type of memory to allocate
    ///
    /// Filter must take memory description and bitmask for all requested buffers
    ///
    /// Bitmask is calculated as bitwise and between all single buffer's bitmasks
    ///
    /// More in [Vulkan spec](https://registry.khronos.org/vulkan/specs/latest/html/vkspec.html#VkMemoryRequirements)
    ///
    /// *Note:* it is your responsibility for memory validation (and checking bitmask in particular)
    ///
    /// Generic filter may look like
    ///
    /// ```rust
    /// use libvktypes::hw;
    ///
    /// fn filter(m: &hw::MemoryDescription, mask: u32) -> bool {
    ///     // take notice how we choose memory type
    ///     let property = hw::MemoryProperty::DEVICE_LOCAL;
    ///
    ///     (mask >> m.index() & 1) == 1 && m.is_compatible(property)
    /// }
    /// ```
    pub fn allocate<'a : 'b, 'b>(
        device: &'a dev::Device,
        cfgs: &mut dyn Iterator<Item = &LayoutElementCfg>,
        filter: &dyn Fn(&hw::MemoryDescription, u32) -> bool,
    ) -> Result<Memory, memory::MemoryError> {
        let mut layout = memory::layout::Layout::new(device, cfgs)?;

        let memory_filter = |m: &'a hw::MemoryDescription| -> Option<&'b hw::MemoryDescription> {
            if filter(m, layout.memory_bits) {
                Some(m)
            } else {
                None
            }
        };

        let mem_desc = on_option_ret!(device.hw().memory().find_map(memory_filter), memory::MemoryError::NoSuitableMemory);

        let dev_memory = memory::Region::allocate(device, layout.alloc_size, mem_desc)?;

        layout.bind(dev_memory.memory())?;

        Ok(Memory {
            i_core: device.core().clone(),
            i_layout: layout,
            i_memory: dev_memory
        })
    }

    pub(crate) fn preallocated(
        core: &Arc<dev::Core>,
        image: vk::Image,
        img_format: vk::Format,
        extent: memory::Extent2D
    ) -> Result<Memory, memory::MemoryError> {
        let iw_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageViewCreateFlags::empty(),
            view_type: vk::ImageViewType::TYPE_2D,
            format: img_format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            image: image,
            _marker: PhantomData,
        };

        let img_view = on_error_ret!(
            unsafe { core.device().create_image_view(&iw_info, core.allocator()) },
            memory::MemoryError::ImageView);

        let requirements = unsafe {
            core
            .device()
            .get_image_memory_requirements(image)
        };

        let image_element = vec![LayoutElement::Image {
            vk_image: image,
            vk_image_view: img_view,
            extent: Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            },
            subresource: vk::ImageSubresourceRange {
                aspect_mask: ImageAspect::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            format: img_format,
            offset: 0,
            allocated_size: requirements.size,
            is_swapchain_image: true
        }];

        let layout = Layout {
            core: core.clone(),
            elements: image_element,
            requested_size: requirements.size,
            alloc_size: requirements.size,
            memory_bits: requirements.memory_type_bits
        };

        Ok(Memory {
            i_core: core.clone(),
            i_layout: layout,
            i_memory: memory::Region::empty(core, requirements.size)
        })
    }

    /// Perfrom operation `f` over selected buffer
    ///
    /// It is relatively expensive operation as memory will be mapped and unmapped
    ///
    /// It is better to use [`map_memory`](Self::map_memory) for frequent changes
    pub fn access<T, F>(&self, f: &mut F, index: usize) -> Result<(), memory::MemoryError>
    where
        F: FnMut(&mut [T]),
    {
        self.i_memory.access(
            f,
            self.i_layout.offset(index),
            self.i_layout.size(index),
            self.i_layout.allocated_size(index)
        )
    }

    /// Return whole size of the memory in bytes
    pub fn size(&self) -> u64 {
        self.i_memory.size()
    }

    /// Map the whole memory into buffer
    pub fn map_memory<T>(&self) -> Result<&mut [T], memory::MemoryError> {
        self.i_memory.map_memory(0, self.i_memory.size(), self.i_memory.size())
    }

    /// Unmap the **whole** memory
    ///
    /// After this call any pointer acquired by [`Memory::map_memory`](Self::map_memory)
    /// or [`View::map_memory`](memory::View::map_memory)
    /// will be invalid
    ///
    /// You **must not** use such pointer
    pub fn unmap_memory(&self) {
        self.i_memory.unmap_memory();
    }

    /// Make host memory changes visible to the device
    ///
    /// Memory **must be** HOST_VISIBLE and **must not be** HOST_COHERENT
    pub fn flush(&self) -> Result<(), memory::MemoryError> {
        self.i_memory.flush(0, self.i_memory.size())
    }

    /// Make device memory changes visible to the host
    ///
    /// Potential use cases are discussed
    /// [here](https://stackoverflow.com/questions/75324067/difference-between-vkinvalidatemappedmemoryranges-and-vkcmdpipelinebarrier-in-vu)
    pub fn sync(&self) -> Result<(), memory::MemoryError> {
        self.i_memory.sync(0, self.i_memory.size())
    }

    pub(crate) fn layout(&self) -> &memory::Layout {
        &self.i_layout
    }

    pub(crate) fn region(&self) -> &memory::Region {
        &self.i_memory
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Memory")
        .field("i_core", &self.i_core)
        .field("i_layout", &self.i_layout)
        .field("i_memory", &self.i_memory)
        .finish()
    }
}

impl fmt::Display for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "core: {:?}\n\
            layout: {:?}\n\
            memory: {:?}\n",
            self.i_core,
            self.i_layout,
            self.i_memory,
        )?;

        Ok(())
    }
}
