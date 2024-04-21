//! Specific version of [`Memory`](crate::memory::Memory) dedicated for rendering

use ash::vk;

use crate::{on_error, on_error_ret};
use crate::{dev, hw, memory};

use std::error::Error;
use std::{fmt, ptr};
use std::sync::Arc;
use std::cell::Cell;

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

/// Errors during [`ImageMemory`] initialization and access
#[derive(Debug)]
pub enum ImageError {
    Creating,
    ImageView,
    DeviceMemory,
    Bind
}

impl fmt::Display for ImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = match self {
            ImageError::Creating => {
                "Failed to create image (vkCreateImage call failed)"
            },
            ImageError::ImageView => {
                "Failed to create image view (vkCreateImageView call failed)"
            },
            ImageError::DeviceMemory => {
                "Failed to allocate memory for image"
            },
            ImageError::Bind => {
                "Failed to bind image memory"
            }
        };

        write!(f, "{:?}", err_msg)
    }
}

impl Error for ImageError {}

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

pub struct ImagesAllocationInfo<'a, 'b : 'a> {
    pub properties: hw::MemoryProperty,
    pub filter: &'a dyn Fn(&hw::MemoryDescription) -> bool,
    pub image_cfgs: &'a [ImageCfg<'b>]
}

#[derive(Debug)]
pub(crate) struct ImageInfo {
    pub extent: Extent3D,
    pub subresource: vk::ImageSubresourceRange,
    pub layout: Cell<memory::ImageLayout>,
    pub format: ImageFormat,
}

impl fmt::Display for ImageInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "extent: {:?}\n\
            aspect: {:?}\n\
            mip level: {:?}\n\
            level count: {:?}\n\
            base array layer: {:?}\n\
            layer count: {:?}\n\
            format: {:?}\n",
            self.extent,
            self.subresource.aspect_mask,
            self.subresource.base_mip_level,
            self.subresource.level_count,
            self.subresource.base_array_layer,
            self.subresource.layer_count,
            self.format
        ).expect("Failed to print ImageInfo");

        Ok(())
    }
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
pub struct ImageMemory {
    i_core: Arc<dev::Core>,
    i_images: Vec<vk::Image>,
    i_image_views: Vec<vk::ImageView>,
    i_subregions: Vec<memory::Subregion>,
    i_info: Vec<ImageInfo>,
    i_memory: memory::Region
}

impl ImageMemory {
    pub fn allocate(device: &dev::Device, cfg: &ImagesAllocationInfo) -> Result<ImageMemory, memory::MemoryError> {
        let mut images: Vec<vk::Image> = Vec::new();
        let mut memory_requirements: Vec<vk::MemoryRequirements> = Vec::new();

        let mut info: Vec<ImageInfo> = Vec::new();

        for cfg in cfg.image_cfgs {
            let sharing_mode = if cfg.simultaneous_access {
                vk::SharingMode::CONCURRENT
            } else {
                vk::SharingMode::EXCLUSIVE
            };

            let image_info = vk::ImageCreateInfo {
                s_type: vk::StructureType::IMAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::ImageCreateFlags::empty(),
                image_type: vk::ImageType::TYPE_2D,
                format: cfg.format,
                extent: cfg.extent,
                mip_levels: 1,
                array_layers: 1,
                samples: vk::SampleCountFlags::TYPE_1,
                tiling: cfg.tiling,
                usage: cfg.usage,
                sharing_mode: sharing_mode,
                queue_family_index_count: cfg.queue_families.len() as u32,
                p_queue_family_indices: cfg.queue_families.as_ptr(),
                initial_layout: cfg.layout,
            };

            for _ in 0..cfg.count {
                let subres = vk::ImageSubresourceRange {
                    aspect_mask: cfg.aspect,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                };

                let img_info = ImageInfo {
                    extent: cfg.extent,
                    subresource: subres,
                    layout: Cell::new(cfg.layout),
                    format: cfg.format
                };

                info.push(img_info);

                let img = on_error!(
                    unsafe { device.device().create_image(&image_info, device.allocator()) },
                    {
                        free_images(device.core(), &images);
                        return Err(memory::MemoryError::Image)
                    }
                );

                images.push(img);

                let requirements = unsafe {
                    device
                    .device()
                    .get_image_memory_requirements(img)
                };

                memory_requirements.push(requirements);
            }
        }

        let regions_info = memory::Region::calculate_subregions(device, &memory_requirements);

        let mem_desc = match memory::Region::find_memory(device.hw(), regions_info.memory_bits, cfg.properties) {
            Some(val) => val,
            None => {
                free_images(device.core(), &images);
                return Err(memory::MemoryError::NoSuitableMemory)
            },
        };

        let img_memory = match memory::Region::allocate(device, regions_info.total_size, mem_desc) {
            Ok(val) => val,
            Err(err) => {
                free_images(device.core(), &images);
                return Err(err);
            }
        };

        for i in 0..images.len() {
            on_error!(
                unsafe {
                    device
                    .device()
                    .bind_image_memory(images[i], img_memory.memory(), regions_info.subregions[i].offset)
                }, {
                    free_images(device.core(), &images);
                    return Err(memory::MemoryError::ImageBind)
                }
            );
        }

        let views = match create_image_views(device.core(), &images, &info) {
            Ok(val) => val,
            Err(err) => {
                free_images(device.core(), &images);
                return Err(err);
            }
        };

        Ok(
            ImageMemory {
                i_core: device.core().clone(),
                i_images: images,
                i_image_views: views,
                i_subregions: regions_info.subregions,
                i_info: info,
                i_memory: img_memory
            }
        )
    }

    /// Create views for all images within allocation
    pub fn views(&self) -> Vec<memory::ImageView> {
        self.i_images.iter().enumerate().map(|(i, _)| memory::ImageView::new(self, i)).collect()
    }

    /// Create and return view to the selected image buffer
    pub fn view(&self, index: usize) -> memory::ImageView {
        memory::ImageView::new(self, index)
    }

    /// Create and return view to the selected image buffer
    pub fn size(&self) -> u64 {
        self.i_memory.size()
    }

    pub(crate) fn access<T, F>(&self, f: &mut F, index: usize) -> Result<(), memory::MemoryError>
    where
        F: FnMut(&mut [T])
    {
        self.i_memory.access(
            f,
            self.i_subregions[index].offset,
            self.i_subregions[index].allocated_size,
            self.i_subregions[index].allocated_size
        )
    }

    pub(crate) fn subregions(&self) -> &Vec<memory::Subregion> {
        &self.i_subregions
    }

    pub(crate) fn image_views(&self) -> &Vec<vk::ImageView> {
        &self.i_image_views
    }

    pub(crate) fn info(&self) -> &Vec<ImageInfo> {
        &self.i_info
    }

    pub(crate) fn images(&self) -> &Vec<vk::Image> {
        &self.i_images
    }

    pub(crate) fn preallocated(
        core: &Arc<dev::Core>,
        image: vk::Image,
        img_format: vk::Format,
        extent: memory::Extent2D
    ) -> Result<ImageMemory, memory::MemoryError> {
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
        };

        let img_view = on_error_ret!(
            unsafe { core.device().create_image_view(&iw_info, core.allocator()) },
            memory::MemoryError::ImageView);

        let requirements = unsafe {
            core
            .device()
            .get_image_memory_requirements(image)
        };

        let img_region = memory::Subregion {
            offset: 0,
            allocated_size: requirements.size
        };

        let img_info = ImageInfo {
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
            layout: Cell::new(memory::ImageLayout::PRESENT_SRC_KHR),
            format: img_format
        };

        Ok(ImageMemory {
            i_core: core.clone(),
            i_images: vec![image],
            i_image_views: vec![img_view],
            i_subregions: vec![img_region],
            i_info: vec![img_info],
            i_memory: memory::Region::empty(core, requirements.size)
        })
    }
}

impl Drop for ImageMemory {
    fn drop(&mut self) {
        free_image_views(&self.i_core, &self.i_image_views);

        if !self.i_memory.is_empty() {
            free_images(&self.i_core, &self.i_images);
        }
    }
}

impl fmt::Debug for ImageMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Memory")
        .field("i_core", &self.i_core)
        .field("i_memory", &self.i_memory)
        .field("i_info", &self.i_info)
        .finish()
    }
}

impl fmt::Display for ImageMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "core: {:?}\n\
            memory: {:?}\n",
            self.i_core,
            self.i_memory
        ).expect("Failed to print Memory");

        for i in 0..self.i_info.len() {
            write!(f,
                "---------------\n\
                id: {:?}\n\
                {:?}",
                i,
                self.i_info[i]
            ).expect("Failed to print Memory");
        }

        Ok(())
    }
}

fn free_images(core: &Arc<dev::Core>, images: &Vec<vk::Image>) {
    for &image in images {
        unsafe {
            core
            .device()
            .destroy_image(image, core.allocator());
        }
    }
}

fn free_image_views(core: &Arc<dev::Core>, images: &Vec<vk::ImageView>) {
    for &image in images {
        unsafe {
            core
            .device()
            .destroy_image_view(image, core.allocator());
        }
    }
}

fn create_image_views(core: &Arc<dev::Core>, images: &Vec<vk::Image>, cfgs: &[ImageInfo])
    -> Result<Vec<vk::ImageView>, memory::MemoryError>
{
    let mut views: Vec<vk::ImageView> = Vec::new();

    for (&img, cfg) in images.iter().zip(cfgs.iter()) {
        let iw_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageViewCreateFlags::empty(),
            view_type: vk::ImageViewType::TYPE_2D,
            format: cfg.format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            },
            subresource_range: cfg.subresource,
            image: img,
        };

        let img_view = on_error!(
            unsafe { core.device().create_image_view(&iw_info, core.allocator()) },
            {
                free_image_views(core, &views);
                return Err(memory::MemoryError::ImageView)
            }
        );

        views.push(img_view);
    }

    Ok(views)
}