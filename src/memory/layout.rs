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

/// Configuration struct for memory region
#[derive(Debug, Clone)]
pub struct BufferCfg<'a> {
    // Size in bytes
    pub size: u64,
    pub usage: BufferUsageFlags,
    pub queue_families: &'a [u32],
    /// Will two or more queues have access to the buffer at the same time
    pub simultaneous_access: bool,
    /// How many of this buffer you want to allocate one by one
    ///
    /// For example
    /// `[<buffer cfg, count == 1>, <buffer cfg, count == 1>]` is equivalent to `[<buffer cfg, count == 2>]`
    ///
    /// Hence each buffer will be handled separately (e.g. for alignment)
    pub count: usize
}

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
pub struct ImageCfg<'a> {
    /// What queue families will have access to the image
    pub queue_families: &'a [u32],
    /// Will two or more queues have access to the buffer at the same time
    pub simultaneous_access: bool,
    pub format: ImageFormat,
    pub extent: Extent3D,
    pub usage: ImageUsageFlags,
    pub layout: memory::ImageLayout,
    pub aspect: ImageAspect,
    pub tiling: Tiling,
    /// How many of the image buffers we want to allocate one by one
    ///
    /// For example
    /// `[<image cfg, count == 1>, <image cfg, count == 1>]` is equivalent to `[<image cfg, count == 2>]`
    ///
    /// Hence each image buffer will be handled separately (e.g. for alignment)
    pub count: usize
}

pub enum LayoutElementCfg<'a> {
    Buffer(BufferCfg<'a>),
    Image(ImageCfg<'a>)
}

#[derive(Debug, Default)]
pub(crate) struct BufferElement {
    vk_buffer: vk::Buffer,
    offset: u64,
    allocated_size: u64,
    size: u64,
}

impl fmt::Display for BufferElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "vk_buffer: {:?}\n\
            offset: {:?}\n\
            allocated_size: {:?}\n\
            size: {:?}\n",
            self.vk_buffer,
            self.offset,
            self.allocated_size,
            self.size
        )?;

        Ok(())
    }
}

#[derive(Debug, Default)]
pub(crate) struct ImageElement {
    pub vk_image: vk::Image,
    pub vk_image_view: vk::ImageView,
    pub extent: Extent3D,
    pub subresource: vk::ImageSubresourceRange,
    pub format: ImageFormat,
    pub offset: u64,
    pub allocated_size: u64,
    pub is_swapchain_image : bool
}

impl fmt::Display for ImageElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "vk_image: {:?}\n\
            vk_image_view: {:?}\n\
            extent: {:?}\n\
            subresource: {:?}\n\
            format: {:?}\n\
            offset: {:?}\n\
            allocated_size: {:?}\n",
            self.vk_image,
            self.vk_image_view,
            self.extent,
            self.subresource,
            self.format,
            self.offset,
            self.allocated_size
        )?;

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) enum LayoutElement {
    Buffer(BufferElement),
    Image(ImageElement)
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
                LayoutElementCfg::Buffer(buffer_cfg) => {
                    let sharing_mode = if buffer_cfg.simultaneous_access {
                        vk::SharingMode::CONCURRENT
                    } else {
                        vk::SharingMode::EXCLUSIVE
                    };

                    let buffer_info = vk::BufferCreateInfo {
                        s_type: vk::StructureType::BUFFER_CREATE_INFO,
                        p_next: std::ptr::null(),
                        flags: vk::BufferCreateFlags::empty(),
                        size: buffer_cfg.size,
                        usage: buffer_cfg.usage,
                        sharing_mode: sharing_mode,
                        queue_family_index_count: buffer_cfg.queue_families.len() as u32,
                        p_queue_family_indices: buffer_cfg.queue_families.as_ptr(),
                        _marker: std::marker::PhantomData,
                    };

                    for _ in 0..buffer_cfg.count {
                        requested_size += buffer_cfg.size;

                        let buffer = on_error!(unsafe {
                            device.device().create_buffer(&buffer_info, device.allocator())
                        }, {
                            free_elements(device.core(), &elements);
                            return Err(memory::MemoryError::Buffer);
                        });

                        let element = BufferElement {
                            vk_buffer: buffer,
                            offset: 0,
                            allocated_size: 0,
                            size: buffer_cfg.size
                        };

                        elements.push(LayoutElement::Buffer(element));

                        let requirements: vk::MemoryRequirements = unsafe {
                            device
                            .device()
                            .get_buffer_memory_requirements(buffer)
                        };

                        memory_requirements.push(requirements);
                    }
                },
                LayoutElementCfg::Image(image_cfg) => {
                    let sharing_mode = if image_cfg.simultaneous_access {
                        vk::SharingMode::CONCURRENT
                    } else {
                        vk::SharingMode::EXCLUSIVE
                    };

                    let image_info = vk::ImageCreateInfo {
                        s_type: vk::StructureType::IMAGE_CREATE_INFO,
                        p_next: std::ptr::null(),
                        flags: vk::ImageCreateFlags::empty(),
                        image_type: vk::ImageType::TYPE_2D,
                        format: image_cfg.format,
                        extent: image_cfg.extent,
                        mip_levels: 1,
                        array_layers: 1,
                        samples: vk::SampleCountFlags::TYPE_1,
                        tiling: image_cfg.tiling,
                        usage: image_cfg.usage,
                        sharing_mode: sharing_mode,
                        queue_family_index_count: image_cfg.queue_families.len() as u32,
                        p_queue_family_indices: image_cfg.queue_families.as_ptr(),
                        initial_layout: image_cfg.layout,
                        _marker: std::marker::PhantomData,
                    };

                    for _ in 0..image_cfg.count {
                        let subres = vk::ImageSubresourceRange {
                            aspect_mask: image_cfg.aspect,
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

                        let img_elem = ImageElement {
                            vk_image: img,
                            vk_image_view: vk::ImageView::null(),
                            extent: image_cfg.extent,
                            subresource: subres,
                            format: image_cfg.format,
                            offset: 0,
                            allocated_size: 0,
                            is_swapchain_image: false
                        };

                        elements.push(LayoutElement::Image(img_elem));

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
                LayoutElement::Buffer(ref mut buff) => {
                    buff.offset = info.offset;
                    buff.allocated_size = info.allocated_size;
                },
                LayoutElement::Image(ref mut img) => {
                    img.allocated_size = info.allocated_size;
                    img.offset = info.offset;

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
                LayoutElement::Buffer(ref buff) => {
                    on_error_ret!(
                        unsafe {
                            self
                            .core
                            .device()
                            .bind_buffer_memory(buff.vk_buffer, memory, buff.offset)
                        },
                        memory::MemoryError::Bind
                    );
                },
                LayoutElement::Image(ref mut img) => {
                    on_error_ret!(
                        unsafe {
                            self
                            .core
                            .device()
                            .bind_image_memory(img.vk_image, memory, img.offset)
                        },
                        memory::MemoryError::ImageBind
                    );

                    let iw_info = vk::ImageViewCreateInfo {
                        s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                        p_next: std::ptr::null(),
                        flags: vk::ImageViewCreateFlags::empty(),
                        view_type: vk::ImageViewType::TYPE_2D,
                        format: img.format,
                        components: vk::ComponentMapping {
                            r: vk::ComponentSwizzle::R,
                            g: vk::ComponentSwizzle::G,
                            b: vk::ComponentSwizzle::B,
                            a: vk::ComponentSwizzle::A,
                        },
                        subresource_range: img.subresource,
                        image: img.vk_image,
                        _marker: std::marker::PhantomData,
                    };

                    img.vk_image_view = on_error_ret!(
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
            LayoutElement::Buffer(buff) => {
                buff.size
            },
            LayoutElement::Image(img) => {
                img.allocated_size
            }
        }
    }

    pub(crate) fn offset(&self, i: usize) -> u64 {
        match &self.elements[i] {
            LayoutElement::Buffer(buff) => {
                buff.offset
            },
            LayoutElement::Image(img) => {
                img.offset
            }
        }
    }

    pub(crate) fn allocated_size(&self, i: usize) -> u64 {
        match &self.elements[i] {
            LayoutElement::Buffer(buff) => {
                buff.allocated_size
            },
            LayoutElement::Image(img) => {
                img.allocated_size
            }
        }
    }

    pub(crate) fn buffer(&self, i: usize) -> vk::Buffer {
        match &self.elements[i] {
            LayoutElement::Buffer(buff) => {
                buff.vk_buffer
            },
            LayoutElement::Image(_) => {
                panic!("Wrong memory element. Expected Buffer found Image");
            }
        }
    }

    pub(crate) fn image(&self, i: usize) -> vk::Image {
        match &self.elements[i] {
            LayoutElement::Buffer(_) => {
                panic!("Wrong memory element. Expected Image found Buffer");
            },
            LayoutElement::Image(img) => {
                img.vk_image
            }
        }
    }

    pub(crate) fn image_view(&self, i: usize) -> vk::ImageView {
        match &self.elements[i] {
            LayoutElement::Buffer(_) => {
                panic!("Wrong memory element. Expected Image found Buffer");
            },
            LayoutElement::Image(img) => {
                img.vk_image_view
            }
        }
    }

    pub(crate) fn extent(&self, i: usize) -> vk::Extent3D {
        match &self.elements[i] {
            LayoutElement::Buffer(_) => {
                panic!("Wrong memory element. Expected Image found Buffer");
            },
            LayoutElement::Image(img) => {
                img.extent
            }
        }
    }

    pub(crate) fn subresource(&self, i: usize) -> vk::ImageSubresourceRange {
        match &self.elements[i] {
            LayoutElement::Buffer(_) => {
                panic!("Wrong memory element. Expected Image found Buffer");
            },
            LayoutElement::Image(img) => {
                img.subresource
            }
        }
    }

    pub(crate) fn format(&self, i: usize) -> vk::Format {
        match &self.elements[i] {
            LayoutElement::Buffer(_) => {
                panic!("Wrong memory element. Expected Image found Buffer");
            },
            LayoutElement::Image(img) => {
                img.format
            }
        }
    }

    pub(crate) fn subresource_layer(&self, i: usize) -> vk::ImageSubresourceLayers {
        match &self.elements[i] {
            LayoutElement::Buffer(_) => {
                panic!("Wrong memory element. Expected Image found Buffer");
            },
            LayoutElement::Image(img) => {
                let subres = img.subresource;

                vk::ImageSubresourceLayers {
                    aspect_mask: subres.aspect_mask,
                    mip_level: subres.base_mip_level,
                    base_array_layer: subres.base_array_layer,
                    layer_count: subres.layer_count
                }
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
                LayoutElement::Buffer(ref buff) => {
                    write!(f,
                        "type: buffer\nindex: {:?}\n{:?}", i, buff)?;
                },
                LayoutElement::Image(ref img) => {
                    write!(f, "type: image\nindex: {:?}\n{:?}", i, img)?;
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
            LayoutElement::Buffer(buffer) => unsafe {
                vk_device.destroy_buffer(buffer.vk_buffer, vk_allocator);
            },
            LayoutElement::Image(image) => unsafe {
                if !image.is_swapchain_image {
                    vk_device.destroy_image(image.vk_image, vk_allocator);
                    vk_device.destroy_image_view(image.vk_image_view, vk_allocator);
                }
                else {
                    vk_device.destroy_image_view(image.vk_image_view, vk_allocator);
                }
            }
        }
    }
}
