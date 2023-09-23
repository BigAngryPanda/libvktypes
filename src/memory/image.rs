//! Specific version of [`Memory`](crate::memory::Memory) dedicated for rendering

use ash::vk;

use crate::{on_error, on_error_ret};
use crate::{dev, graphics, hw, memory};

use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::ptr;

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
    pub layout: graphics::ImageLayout,
    pub aspect: ImageAspect,
    pub tiling: Tiling
}

pub struct ImagesAllocationInfo<'a, 'b : 'a> {
    pub properties: hw::MemoryProperty,
    pub filter: &'a dyn Fn(&hw::MemoryDescription) -> bool,
    pub image_cfgs: &'a [ImageCfg<'b>]
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
    i_extents: Vec<Extent3D>,
    i_memory: memory::Region
}

impl ImageMemory {
    pub fn allocate(device: &dev::Device, cfg: &ImagesAllocationInfo) -> Result<ImageMemory, memory::MemoryError> {
        let mut images: Vec<vk::Image> = Vec::new();
        let mut memory_requirements: Vec<vk::MemoryRequirements> = Vec::new();
        let mut extents: Vec<Extent3D> = Vec::new();

        for cfg in cfg.image_cfgs {
            extents.push(cfg.extent);

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

        let views = match create_image_views(device.core(), &images, cfg.image_cfgs) {
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
                i_extents: extents,
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
            self.i_subregions[index].allocated_size
        )
    }

    pub(crate) fn subregions(&self) -> &Vec<memory::Subregion> {
        &self.i_subregions
    }

    pub(crate) fn image_views(&self) -> &Vec<vk::ImageView> {
        &self.i_image_views
    }

    pub(crate) fn extents(&self) -> &Vec<Extent3D> {
        &self.i_extents
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

        Ok(ImageMemory {
            i_core: core.clone(),
            i_images: vec![image],
            i_image_views: vec![img_view],
            i_subregions: vec![img_region],
            i_extents: vec![Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            }],
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
        .field("i_images", &self.i_images)
        .field("i_image_views", &self.i_image_views)
        .field("i_subregions", &self.i_subregions)
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

        for i in 0..self.i_subregions.len() {
            write!(f,
                "---------------\n\
                id: {:?}\n\
                image {:?}\n\
                image view {:?}\n\
                subregion: {:?}\n",
                i,
                self.i_images[i],
                self.i_image_views[i],
                self.i_subregions[i]
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

fn create_image_views(core: &Arc<dev::Core>, images: &Vec<vk::Image>, cfg: &[ImageCfg])
    -> Result<Vec<vk::ImageView>, memory::MemoryError>
{
    let mut views: Vec<vk::ImageView> = Vec::new();

    for i in 0..images.len() {
        let iw_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageViewCreateFlags::empty(),
            view_type: vk::ImageViewType::TYPE_2D,
            format: cfg[i].format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: cfg[i].aspect,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            image: images[i],
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