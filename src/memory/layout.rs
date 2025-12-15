use ash::vk;

use crate::{
    memory,
    dev,
    on_error,
    on_error_ret,
    offset
};

use std::fmt;

/// Purpose of buffer
///
#[doc = "Ash documentation about possible values <https://docs.rs/ash/latest/ash/vk/struct.BufferUsageFlags.html>"]
///
#[doc = "Vulkan documentation <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkBufferUsageFlagBits.html>"]
pub type BufferUsageFlags = vk::BufferUsageFlags;

/// Represents image usage flags
///
#[doc = "Possible values: <https://docs.rs/ash/latest/ash/vk/struct.ImageUsageFlags.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkImageUsageFlagBits.html>"]
pub type ImageUsageFlags = vk::ImageUsageFlags;

/// Represents which aspects of an image will be used
///
#[doc = "Possible values: <https://docs.rs/ash/latest/ash/vk/struct.ImageAspectFlags.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkImageAspectFlagBits.html>"]
pub type ImageAspect = vk::ImageAspectFlags;

/// Image formats
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.Format.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkFormat.html>"]
pub type ImageFormat = vk::Format;

/// Structure specifying a two-dimensional extent
///
/// Contains two field: `width` and `height`
///
#[doc = "Ash documentation: <https://docs.rs/ash/latest/ash/vk/struct.Extent2D.html>"]
///
#[doc = "Vulkan documentation: <https://docs.rs/ash/latest/ash/vk/struct.Extent2D.html>"]
///
/// # Example
///
/// ```
/// use libvktypes::memory::Extent2D;
///
/// Extent2D {
///     width: 1920,
///     height: 1080,
/// };
/// ```
pub type Extent2D = vk::Extent2D;

/// Structure specifying a three-dimensional extent
///
#[doc = "Ash documentation: <https://docs.rs/ash/latest/ash/vk/struct.Extent3D.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkExtent3D.html>"]
///
/// # Example
///
/// ```
/// use libvktypes::memory::Extent3D;
///
/// Extent3D {
///     width: 1920,
///     height: 1080,
///     depth: 1,
/// };
/// ```
pub type Extent3D = vk::Extent3D;

/// Image usage flags
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.ImageUsageFlags.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkImageUsageFlagBits.html>"]
pub type UsageFlags = vk::ImageUsageFlags;

/// Color spaces
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.ColorSpaceKHR.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkColorSpaceKHR.html>"]
pub type ColorSpace = vk::ColorSpaceKHR;

/// Value indicating the alpha compositing mode to use when this surface is composited together with other surfaces on certain window systems
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.CompositeAlphaFlagsKHR.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkCompositeAlphaFlagBitsKHR.html>"]
pub type CompositeAlphaFlags = vk::CompositeAlphaFlagsKHR;

/// Specifying the tiling arrangement of texel blocks in an image
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.ImageTiling.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkImageTiling.html>"]
pub type Tiling = vk::ImageTiling;

/// Buffer represents generic (except images) configuration struct for memory region
///
/// Images represent multidimensional - up to 3 - arrays of data
///
/// # Access
///
/// [`ImageView`](crate::memory::ImageView::access) provides access to the memory region
///
/// Memory will be mapped and unmapped each time
///
/// # Preallocated memory
///
/// [`Swapchain::images`](crate::swapchain::Swapchain::images) provides already allocated images
/// so no extra allocation will be performed
///
/// However as [`ImageMemory`] does not own image memory
/// you must not call [`access`](crate::memory::ImageView::access) on such images
///
/// Nonetheless size of such memory may be non-zero
///
/// # Image size
///
/// As you do not explicitly define how many bytes image size will be allocated size is used as requested size
///
/// It is important for [`map_memory`](memory::ImageView::map_memory) function as you have to take into account
/// returned buffer may be larger that you are expecting
#[derive(Debug)]
pub enum LayoutElementCfg<'a> {
    Buffer {
        // Size in bytes
        size: u64,
        usage: BufferUsageFlags,
        queue_families: &'a [u32],
        /// Will two or more queues have access to the buffer at the same time
        simultaneous_access: bool,
        /// How many of this buffer you want to allocate one by one
        ///
        /// For example
        /// `[<buffer cfg, count == 1>, <buffer cfg, count == 1>]` is equivalent to `[<buffer cfg, count == 2>]`
        ///
        /// Hence each buffer will be handled separately (e.g. for alignment)
        count: usize
    },
    Image {
        /// What queue families will have access to the image
        queue_families: &'a [u32],
        /// Will two or more queues have access to the buffer at the same time
        simultaneous_access: bool,
        format: ImageFormat,
        extent: Extent3D,
        usage: ImageUsageFlags,
        layout: memory::ImageLayout,
        aspect: ImageAspect,
        tiling: Tiling,
        /// How many of the image buffers we want to allocate one by one
        ///
        /// For example
        /// `[<image cfg, count == 1>, <image cfg, count == 1>]` is equivalent to `[<image cfg, count == 2>]`
        ///
        /// Hence each image buffer will be handled separately (e.g. for alignment)
        count: usize
    }
}

#[derive(Debug)]
pub(crate) enum LayoutElement {
    Buffer {
        vk_buffer: vk::Buffer,
        offset: u64,
        allocated_size: u64,
        size: u64,
    },
    Image {
        vk_image: vk::Image,
        vk_image_view: vk::ImageView,
        extent: Extent3D,
        subresource: vk::ImageSubresourceRange,
        format: ImageFormat,
        offset: u64,
        allocated_size: u64,
        is_swapchain_image : bool
    }
}

#[derive(Debug)]
pub(crate) struct Layout {
    pub core: std::sync::Arc<dev::Core>,
    pub elements: Vec<LayoutElement>,
    pub requested_size: u64,
    pub alloc_size: u64,
    pub memory_bits: u32
}

impl Layout {
    pub(crate) fn new(
        device: &dev::Device,
        cfgs: &mut dyn Iterator<Item = &LayoutElementCfg>) -> Result<Layout, memory::MemoryError>
    {
        let mut memory_requirements: Vec<vk::MemoryRequirements> = Vec::new();
        let mut elements: Vec<LayoutElement> = Vec::new();

        let mut requested_size: u64 = 0;

        for cfg in cfgs {
            match cfg {
                &LayoutElementCfg::Buffer { size, usage, queue_families, simultaneous_access, count } => {
                    let sharing_mode = if simultaneous_access {
                        vk::SharingMode::CONCURRENT
                    } else {
                        vk::SharingMode::EXCLUSIVE
                    };

                    let buffer_info = vk::BufferCreateInfo {
                        s_type: vk::StructureType::BUFFER_CREATE_INFO,
                        p_next: std::ptr::null(),
                        flags: vk::BufferCreateFlags::empty(),
                        size: size,
                        usage: usage,
                        sharing_mode: sharing_mode,
                        queue_family_index_count: queue_families.len() as u32,
                        p_queue_family_indices: queue_families.as_ptr(),
                        _marker: std::marker::PhantomData,
                    };

                    for _ in 0..count {
                        requested_size += size;

                        let buffer = on_error!(unsafe {
                            device.device().create_buffer(&buffer_info, device.allocator())
                        }, {
                            free_elements(device.core(), &elements);
                            return Err(memory::MemoryError::Buffer);
                        });

                        let element = LayoutElement::Buffer {
                            vk_buffer: buffer,
                            offset: 0,
                            allocated_size: 0,
                            size
                        };

                        elements.push(element);

                        let requirements: vk::MemoryRequirements = unsafe {
                            device
                            .device()
                            .get_buffer_memory_requirements(buffer)
                        };

                        memory_requirements.push(requirements);
                    }
                },
                &LayoutElementCfg::Image {
                    queue_families,
                    simultaneous_access,
                    format,
                    extent,
                    usage,
                    layout,
                    aspect,
                    tiling,
                    count
                } => {
                    let sharing_mode = if simultaneous_access {
                        vk::SharingMode::CONCURRENT
                    } else {
                        vk::SharingMode::EXCLUSIVE
                    };

                    let image_info = vk::ImageCreateInfo {
                        s_type: vk::StructureType::IMAGE_CREATE_INFO,
                        p_next: std::ptr::null(),
                        flags: vk::ImageCreateFlags::empty(),
                        image_type: vk::ImageType::TYPE_2D,
                        format: format,
                        extent: extent,
                        mip_levels: 1,
                        array_layers: 1,
                        samples: vk::SampleCountFlags::TYPE_1,
                        tiling: tiling,
                        usage: usage,
                        sharing_mode: sharing_mode,
                        queue_family_index_count: queue_families.len() as u32,
                        p_queue_family_indices: queue_families.as_ptr(),
                        initial_layout: layout,
                        _marker: std::marker::PhantomData,
                    };

                    for _ in 0..count {
                        let subres = vk::ImageSubresourceRange {
                            aspect_mask: aspect,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        };

                        let img = on_error!(
                            unsafe { device.device().create_image(&image_info, device.allocator()) },
                            {
                                free_elements(device.core(), &elements);
                                return Err(memory::MemoryError::Image)
                            }
                        );

                        let img_elem = LayoutElement::Image {
                            vk_image: img,
                            vk_image_view: vk::ImageView::null(),
                            extent: extent,
                            subresource: subres,
                            format: format,
                            offset: 0,
                            allocated_size: 0,
                            is_swapchain_image: false
                        };

                        elements.push(img_elem);

                        let requirements = unsafe {
                            device
                            .device()
                            .get_image_memory_requirements(img)
                        };

                        memory_requirements.push(requirements);
                    }
                }
            }
        }

        let regions_info = calculate_subregions(device, &memory_requirements);

        for (elem, info) in elements.iter_mut().zip(regions_info.subregions) {
            match elem {
                LayoutElement::Buffer { ref mut offset, ref mut allocated_size, .. } => {
                    *offset = info.offset;
                    *allocated_size = info.allocated_size;
                },
                LayoutElement::Image { ref mut allocated_size, ref mut offset, .. } => {
                    *allocated_size = info.allocated_size;
                    *offset = info.offset;

                    requested_size += info.allocated_size;
                }
            }
        }

        Ok(Layout {
            core: device.core().clone(),
            elements: elements,
            alloc_size: regions_info.total_size,
            requested_size: requested_size,
            memory_bits: regions_info.memory_bits
        })
    }

    pub(crate) fn bind(&mut self, memory: vk::DeviceMemory) -> Result<(), memory::MemoryError> {
        for elem in &mut self.elements {
            match elem {
                &mut LayoutElement::Buffer { vk_buffer, offset, .. } => {
                    on_error_ret!(
                        unsafe {
                            self
                            .core
                            .device()
                            .bind_buffer_memory(vk_buffer, memory, offset)
                        },
                        memory::MemoryError::Bind
                    );
                },
                &mut LayoutElement::Image { vk_image, offset, format, subresource, ref mut vk_image_view, .. } => {
                    on_error_ret!(
                        unsafe {
                            self
                            .core
                            .device()
                            .bind_image_memory(vk_image, memory, offset)
                        },
                        memory::MemoryError::ImageBind
                    );

                    let iw_info = vk::ImageViewCreateInfo {
                        s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                        p_next: std::ptr::null(),
                        flags: vk::ImageViewCreateFlags::empty(),
                        view_type: vk::ImageViewType::TYPE_2D,
                        format: format,
                        components: vk::ComponentMapping {
                            r: vk::ComponentSwizzle::R,
                            g: vk::ComponentSwizzle::G,
                            b: vk::ComponentSwizzle::B,
                            a: vk::ComponentSwizzle::A,
                        },
                        subresource_range: subresource,
                        image: vk_image,
                        _marker: std::marker::PhantomData,
                    };

                    *vk_image_view = on_error_ret!(
                        unsafe { self.core.device().create_image_view(&iw_info, self.core.allocator()) },
                        memory::MemoryError::ImageView
                    );
                }
            }
        }

        Ok(())
    }

    pub(crate) fn size(&self, i: usize) -> u64 {
        match &self.elements[i] {
            &LayoutElement::Buffer { size, .. } => {
                size
            },
            &LayoutElement::Image { allocated_size, .. } => {
                allocated_size
            }
        }
    }

    pub(crate) fn offset(&self, i: usize) -> u64 {
        match &self.elements[i] {
            &LayoutElement::Buffer { offset, .. } => {
                offset
            },
            &LayoutElement::Image { offset, .. } => {
                offset
            }
        }
    }

    pub(crate) fn allocated_size(&self, i: usize) -> u64 {
        match &self.elements[i] {
            &LayoutElement::Buffer { allocated_size, .. } => {
                allocated_size
            },
            &LayoutElement::Image { allocated_size, .. } => {
                allocated_size
            }
        }
    }

    pub(crate) fn buffer(&self, i: usize) -> vk::Buffer {
        match &self.elements[i] {
            &LayoutElement::Buffer { vk_buffer, .. } => {
                vk_buffer
            },
            _ => {
                panic!("Wrong memory element. Expected Buffer found Image");
            }
        }
    }

    pub(crate) fn image(&self, i: usize) -> vk::Image {
        match &self.elements[i] {
            &LayoutElement::Image { vk_image, .. } => {
                vk_image
            },
            _ => {
                panic!("Wrong memory element. Expected Image found Buffer");
            }
        }
    }

    pub(crate) fn image_view(&self, i: usize) -> vk::ImageView {
        match &self.elements[i] {
            &LayoutElement::Image { vk_image_view, .. } => {
                vk_image_view
            },
            _ => {
                panic!("Wrong memory element. Expected Image found Buffer");
            }
        }
    }

    pub(crate) fn extent(&self, i: usize) -> vk::Extent3D {
        match &self.elements[i] {
            &LayoutElement::Image { extent, .. } => {
                extent
            },
            _ => {
                panic!("Wrong memory element. Expected Image found Buffer");
            }
        }
    }

    pub(crate) fn subresource(&self, i: usize) -> vk::ImageSubresourceRange {
        match &self.elements[i] {
            &LayoutElement::Image { subresource, .. } => {
                subresource
            },
            _ => {
                panic!("Wrong memory element. Expected Image found Buffer");
            }
        }
    }

    pub(crate) fn format(&self, i: usize) -> vk::Format {
        match &self.elements[i] {
            &LayoutElement::Image { format, .. } => {
                format
            },
            _ => {
                panic!("Wrong memory element. Expected Image found Buffer");
            }
        }
    }

    pub(crate) fn subresource_layer(&self, i: usize) -> vk::ImageSubresourceLayers {
        match &self.elements[i] {
            LayoutElement::Image { subresource, .. } => {
                vk::ImageSubresourceLayers {
                    aspect_mask: subresource.aspect_mask,
                    mip_level: subresource.base_mip_level,
                    base_array_layer: subresource.base_array_layer,
                    layer_count: subresource.layer_count
                }
            },
            _ => {
                panic!("Wrong memory element. Expected Image found Buffer");
            }
        }


    }
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "core: {:?}\n\
            requested_size: {:?}\n\
            alloc_size: {:?}\n\
            memory_bits: {:?}\n",
            self.core,
            self.alloc_size,
            self.requested_size,
            self.memory_bits
        )?;

        for (i, elem) in self.elements.iter().enumerate() {
            write!(f, "---------------\n")?;

            match elem {
                LayoutElement::Buffer { vk_buffer, offset, allocated_size, size } => {
                    write!(f,
                        "
                        type: buffer\n\
                        index: {:?}\n\
                        vk_buffer: {:?}\n\
                        offset: {:?}\n\
                        allocated_size: {:?}\n\
                        size: {:?}\n",
                        i,
                        vk_buffer,
                        offset,
                        allocated_size,
                        size
                    )?;
                },
                LayoutElement::Image {
                    vk_image,
                    vk_image_view,
                    extent,
                    subresource,
                    format,
                    offset,
                    allocated_size,
                    is_swapchain_image
                } => {
                    write!(f,
                        "type: image\n\
                        index: {:?}\n\
                        vk_image: {:?}\n\
                        vk_image_view: {:?}\n\
                        extent: {:?}\n\
                        subresource: {:?}\n\
                        format: {:?}\n\
                        offset: {:?}\n\
                        allocated_size: {:?}\n\
                        is_swapchain_image: {:?}",
                        i,
                        vk_image,
                        vk_image_view,
                        extent,
                        subresource,
                        format,
                        offset,
                        allocated_size,
                        is_swapchain_image
                    )?;
                }
            }
        }

        Ok(())
    }
}

impl Drop for Layout {
    fn drop(&mut self) {
        free_elements(&self.core, &self.elements);
    }
}

struct LayoutSubregion {
    pub offset: u64,
    pub allocated_size: u64
}

struct LayoutSizeInfo {
    pub subregions: Vec<LayoutSubregion>,
    pub total_size: u64,
    pub memory_bits: u32
}

fn calculate_subregions(
    device: &dev::Device,
    requirements: &[vk::MemoryRequirements]) -> LayoutSizeInfo
{
    let mut memory_type_bits = 0xffffffffu32;
    let mut last = 0u64;
    let mut total_size = 0u64;
    let mut subregions: Vec<LayoutSubregion> = Vec::new();

    for requirement in requirements {
        // On one hand memory should be aligned for nonCoherentAtomSize
        // On the other side for requirements.alignment
        // So resulting alignment will be hcf(nonCoherentAtomSize, requirements.alignment)
        // Spec states that both of them are power of two so calculation may be reduced
        // To calculating max of the values
        // See https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#limits
        // https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#VkMemoryRequirements
        // https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkMemoryRequirements.html
        //
        // Useful note on the alignment
        // https://stackoverflow.com/questions/51439858/use-correct-offset-when-binding-a-buffer-to-a-memory#51440838
        let alignment = std::cmp::max(device.hw().memory_alignment(), requirement.alignment);

        // How many bytes we need after *previous* buffer
        let begin_offset = offset::padding_bytes(last, alignment);

        // How many bytes we need after *current* buffer
        let end_offset = offset::padding_bytes(requirement.size, alignment);

        let aligned_size = requirement.size + end_offset;

        last += begin_offset;
        subregions.push(LayoutSubregion { offset: last, allocated_size: requirement.size});

        memory_type_bits &= requirement.memory_type_bits;

        last += aligned_size;

        total_size += requirement.size + alignment;
    }

    LayoutSizeInfo {
        subregions,
        total_size,
        memory_bits: memory_type_bits
    }
}

fn free_elements(device: &dev::Core, elements: &Vec<LayoutElement>) {
    use crate::alloc;

    let vk_device: &ash::Device = device.device();
    let vk_allocator: Option<&alloc::Callback> = device.allocator();

    for elem in elements {
        match elem {
            &LayoutElement::Buffer { vk_buffer, .. } => unsafe {
                vk_device.destroy_buffer(vk_buffer, vk_allocator);
            },
            &LayoutElement::Image { vk_image, vk_image_view, is_swapchain_image, .. } => unsafe {
                if !is_swapchain_image {
                    vk_device.destroy_image(vk_image, vk_allocator);
                    vk_device.destroy_image_view(vk_image_view, vk_allocator);
                }
                else {
                    vk_device.destroy_image_view(vk_image_view, vk_allocator);
                }
            }
        }
    }
}
