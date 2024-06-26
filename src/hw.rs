//! Provide information about available hardware
//!
//! Instead of [dev module](crate::dev) `hw` represents hardware level

use ash::vk;

use crate::on_error_ret;
use crate::{libvk, surface, offset};

use std::ffi::CStr;
use std::fmt;

#[derive(Debug)]
pub enum HWError {
    Enumerate,
    SurfaceSupport,
}

/// Represents GPU type
///
#[doc = "Ash documentation about possible values <https://docs.rs/ash/latest/ash/vk/struct.BorderColor.html>"]
///
#[doc = "See more <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPhysicalDeviceType.html>"]
pub type HWType = vk::PhysicalDeviceType;

/// Represent information about single queue family
///
#[doc = "See more <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/html/vkspec.html#VkQueueFlagBits>"]
#[derive(Debug, Clone, Copy)]
pub struct QueueFamilyDescription {
    i_index: u32,
    i_count: u32,
    i_property: vk::QueueFlags,
    i_surface_support: bool,
}

impl QueueFamilyDescription {
    #[doc(hidden)]
    fn new(property: &vk::QueueFamilyProperties, index: u32, hw: vk::PhysicalDevice, surface: Option<&surface::Surface>)
        -> QueueFamilyDescription
    {
        let surface_support = if let Some(val) = surface {
            matches!(
                unsafe {
                    val.loader().get_physical_device_surface_support(
                        hw,
                        index,
                        val.surface(),
                    )
                },
                Ok(true)
            )
        } else {
            false
        };

        QueueFamilyDescription {
            i_index: index,
            i_count: property.queue_count,
            i_property: property.queue_flags,
            i_surface_support: surface_support,
        }
    }

    /// Return how many queues in family
    pub fn count(&self) -> u32 {
        self.i_count
    }

    /// Return queue family index
    pub fn index(&self) -> u32 {
        self.i_index
    }

    /// Is VK_QUEUE_GRAPHICS_BIT set for queue family
    pub fn is_graphics(&self) -> bool {
        self.i_property.contains(vk::QueueFlags::GRAPHICS)
    }

    /// Is VK_QUEUE_COMPUTE_BIT set for queue family
    pub fn is_compute(&self) -> bool {
        self.i_property.contains(vk::QueueFlags::COMPUTE)
    }

    /// Is VK_QUEUE_TRANSFER_BIT set for queue family
    pub fn is_transfer(&self) -> bool {
        self.i_property.contains(vk::QueueFlags::TRANSFER)
    }

    /// Is VK_QUEUE_SPARSE_BINDING_BIT set for queue family
    pub fn is_sparce_binding(&self) -> bool {
        self.i_property.contains(vk::QueueFlags::SPARSE_BINDING)
    }

    /// If [`surface`](crate::surface::Surface) was provided in [`poll`](crate::hw::Description::poll)
    /// returns does selected queue family support `surface`
    ///
    /// Otherwise returns default value: [`false`]
    pub fn is_surface_supported(&self) -> bool {
        self.i_surface_support
    }

    /// Does selected queue family within hw device supports surface
    ///
    /// Return [error](crate::hw::HWError) if failed to get support
    pub fn explicit_support_surface(
        &self,
        hw: &HWDevice,
        surface: &surface::Surface
    ) -> Result<bool, HWError> {
        match unsafe {
            surface.loader().get_physical_device_surface_support(
                hw.device(),
                self.i_index,
                surface.surface(),
            )
        } {
            Ok(val) => Ok(val),
            Err(_) => Err(HWError::SurfaceSupport),
        }
    }

    /// Does selected queue family within hw device supports surface
    ///
    /// Instead of [explicit method](crate::hw::QueueFamilyDescription::explicit_support_surface)
    /// return false if failed to get support or queue family does not support presentation
    pub fn support_surface(
        &self,
        hw: &HWDevice,
        surface: &surface::Surface
    ) -> bool {
        matches!(self.explicit_support_surface(hw, surface), Ok(true))
    }
}

impl fmt::Display for QueueFamilyDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Number of queues:      {}\n\
            Support graphics:       {}\n\
            Support compute:        {}\n\
            Support transfer:       {}\n\
            Support sparce binding: {}\n\
            Support surface:        {}\n",
            self.count(),
            if self.is_graphics() { "yes" } else { "no" },
            if self.is_compute()  { "yes" } else { "no" },
            if self.is_transfer() { "yes" } else { "no" },
            if self.is_sparce_binding() {
                "yes"
            } else {
                "no"
            },
            if self.is_surface_supported() {
                "yes"
            } else {
                "no"
            }
        )
    }
}

/// Represents memory capabilities
///
/// Each memory has its own property as bitmask
///
/// Method checks that selected memory satisfies requirements defined by ```flags```
///
#[doc = "Possible values: <https://docs.rs/ash/latest/ash/vk/struct.MemoryPropertyFlags.html>"]
///
#[doc = "See more <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkMemoryPropertyFlagBits.html>"]
pub type MemoryProperty = vk::MemoryPropertyFlags;

/// Represents information about each heap
///
#[doc = "See more <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPhysicalDeviceMemoryProperties.html>"]
#[derive(Debug, Clone, Copy)]
pub struct MemoryDescription {
    i_mem_index: u32,
    /// Heap size in bytes
    i_heap_size: u64,
    /// Corresponding heap index
    i_heap_index: u32,
    i_property: vk::MemoryPropertyFlags,
}

impl MemoryDescription {
    #[doc(hidden)]
    fn new(properties: &vk::PhysicalDeviceMemoryProperties, mem_index: usize) -> MemoryDescription {
        let mem_type: vk::MemoryType = properties.memory_types[mem_index];
        let heap_size: u64 = properties.memory_heaps[mem_type.heap_index as usize].size;

        MemoryDescription {
            i_mem_index: mem_index as u32,
            i_heap_size: heap_size,
            i_heap_index: mem_type.heap_index,
            i_property: mem_type.property_flags,
        }
    }

    /// Return memory type index
    pub fn index(&self) -> u32 {
        self.i_mem_index
    }

    /// Return heap size in bytes
    pub fn heap_size(&self) -> u64 {
        self.i_heap_size
    }

    /// Return heap index
    pub fn heap_index(&self) -> u32 {
        self.i_heap_index
    }

    /// Each memory has its own property as bitmask
    ///
    /// Method checks that selected memory satisfies requirements defined by ```flags```
    ///
    #[doc = "See more <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkMemoryPropertyFlagBits.html>"]
    pub fn is_compatible(&self, flags: MemoryProperty) -> bool {
        self.i_property.contains(flags)
    }

    /// Is VK_MEMORY_HEAP_DEVICE_LOCAL_BIT set
    pub fn is_local(&self) -> bool {
        self.i_property
            .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
    }

    /// Is VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT set for the memory
    pub fn is_host_visible(&self) -> bool {
        self.i_property
            .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
    }

    /// Is VK_MEMORY_PROPERTY_HOST_COHERENT_BIT set for the memory
    pub fn is_host_coherent(&self) -> bool {
        self.i_property
            .contains(vk::MemoryPropertyFlags::HOST_COHERENT)
    }

    /// Is VK_MEMORY_PROPERTY_HOST_CACHED_BIT set for the memory
    pub fn is_host_cached(&self) -> bool {
        self.i_property
            .contains(vk::MemoryPropertyFlags::HOST_CACHED)
    }

    /// Is VK_MEMORY_PROPERTY_LAZILY_ALLOCATED_BIT set for the memory
    pub fn is_lazily_allocated(&self) -> bool {
        self.i_property
            .contains(vk::MemoryPropertyFlags::LAZILY_ALLOCATED)
    }

    /// Return memory property flags
    pub fn flags(&self) -> MemoryProperty {
        self.i_property
    }
}

impl fmt::Display for MemoryDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mem_size = self.heap_size();

        write!(
            f,
            "Memory type index: {}\n\
            Heap index:        {}\n\
            Heap size: {} bytes, {} kb, {} mb, {} gb\n\
            Memory properties:  \n\
            Local:            {}\n\
            Host visible:     {}\n\
            Host coherent:    {}\n\
            Host cached:      {}\n\
            Lazily allocated: {}\n",
            self.index(),
            self.heap_index(),
            mem_size,
            mem_size / 1024,
            mem_size / (1024 * 1024),
            mem_size / (1024 * 1024 * 1024),
            if self.is_local() { "yes" } else { "no" },
            if self.is_host_visible() { "yes" } else { "no" },
            if self.is_host_coherent() { "yes" } else { "no" },
            if self.is_host_cached() { "yes" } else { "no" },
            if self.is_lazily_allocated() {
                "yes"
            } else {
                "no"
            }
        )
    }
}

pub type Features = vk::PhysicalDeviceFeatures;

#[derive(Clone)]
pub struct HWDevice {
    i_device: vk::PhysicalDevice,
    i_properties: vk::PhysicalDeviceProperties,
    i_features: Features,
    i_queues: Vec<QueueFamilyDescription>,
    i_heap_info: Vec<MemoryDescription>,
}

impl HWDevice {
    fn new(lib: &libvk::Instance, hw: vk::PhysicalDevice, surface: Option<&surface::Surface>)
        -> HWDevice
    {
        let properties: vk::PhysicalDeviceProperties =
            unsafe { lib.instance().get_physical_device_properties(hw) };

        let queue_properties: Vec<vk::QueueFamilyProperties> = unsafe {
            lib.instance()
                .get_physical_device_queue_family_properties(hw)
        };

        let mem_props: vk::PhysicalDeviceMemoryProperties = unsafe {
            lib.instance().get_physical_device_memory_properties(hw)
        };

        let mut memory_desc: Vec<MemoryDescription> = Vec::new();

        for i in 0..mem_props.memory_type_count as usize {
            memory_desc.push(MemoryDescription::new(&mem_props, i));
        }

        let queue_desc: Vec<QueueFamilyDescription> =
            queue_properties
            .iter()
            .enumerate()
            .map(|(i, prop)| QueueFamilyDescription::new(prop, i as u32, hw, surface))
            .filter(|q| {
                q.is_compute() || q.is_graphics() || q.is_transfer() || q.is_sparce_binding()
            })
            .collect();

        HWDevice {
            i_device: hw,
            i_features: unsafe { lib.instance().get_physical_device_features(hw) },
            i_properties: properties,
            i_queues: queue_desc,
            i_heap_info: memory_desc,
        }
    }

    pub(crate) fn device(&self) -> vk::PhysicalDevice {
        self.i_device
    }

    /// Features information
    pub fn features(&self) -> &Features {
        &self.i_features
    }

    /// Device name
    pub fn name(&self) -> String {
        unsafe {
            CStr::from_ptr(&self.i_properties.device_name[0])
                .to_str()
                .unwrap()
                .to_owned()
        }
    }

    /// Return device type
    pub fn device_type(&self) -> HWType {
        self.i_properties.device_type
    }

    /// Hardware id
    pub fn hw_id(&self) -> u32 {
        self.i_properties.device_id
    }

    /// Return packed version
    ///
    #[doc = "About version <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/html/vkspec.html#extendingvulkan-coreversions-versionnumbers>"]
    pub fn version(&self) -> u32 {
        self.i_properties.api_version
    }

    /// Return API major version number
    ///
    #[doc = "About version <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VK_API_VERSION_MAJOR.html>"]
    pub fn version_major(&self) -> u32 {
        vk::api_version_major(self.version())
    }

    /// Return API minor version number
    ///
    #[doc = "About version <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VK_API_VERSION_MINOR.html>"]
    pub fn version_minor(&self) -> u32 {
        vk::api_version_minor(self.version())
    }

    /// Return API patch version number
    ///
    #[doc = "About version <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VK_API_VERSION_PATCH.html>"]
    pub fn version_patch(&self) -> u32 {
        vk::api_version_patch(self.version())
    }

    /// Return vendor id
    pub fn vendor_id(&self) -> u32 {
        self.i_properties.vendor_id
    }

    /// Return true if GPU type is `Discrete`
    ///
    /// Otherwise false
    ///
    /// See HWType
    pub fn is_discrete_gpu(&self) -> bool {
        self.device_type() == HWType::DISCRETE_GPU
    }

    /// Return true if GPU type is `Integrated`
    ///
    /// Otherwise false
    ///
    /// See HWType
    pub fn is_integrated_gpu(&self) -> bool {
        self.device_type() == HWType::INTEGRATED_GPU
    }

    /// Return true if GPU type is `Integrated` or `Discrete`
    ///
    /// Otherwise false
    ///
    /// See [`HWType`]
    pub fn is_dedicated_gpu(&self) -> bool {
        self.is_discrete_gpu() || self.is_integrated_gpu()
    }

    /// Minimal offset for uniform buffer binding
    pub fn ubo_offset(&self) -> u64 {
        self.i_properties.limits.min_uniform_buffer_offset_alignment
    }

    /// Calculate buffer size with respect for dynamic alignment
    ///
    /// For 0 sized buffer 0 will be returned
    ///
    /// This method is useful when you have to deal with dynamic ubo
    pub fn ubo_size(&self, requested_size: u64) -> u64 {
        offset::full_size(requested_size, self.ubo_offset())
    }

    /// Minimal offset for storage buffer binding
    pub fn storage_offset(&self) -> u64 {
        self.i_properties.limits.min_storage_buffer_offset_alignment
    }

    /// Memory mapping alignment
    pub fn memory_alignment(&self) -> u64 {
        self.i_properties.limits.non_coherent_atom_size
    }

    /// Max sampler anisotropy
    pub fn max_anisotropy(&self) -> f32 {
        self.i_properties.limits.max_sampler_anisotropy
    }

    /// Return iterator over available queues
    pub fn queues(&self) -> impl Iterator<Item = &QueueFamilyDescription> {
        self.i_queues.iter()
    }

    /// Return iterator over available memory heaps
    pub fn memory(&self) -> impl Iterator<Item = &MemoryDescription> {
        self.i_heap_info.iter()
    }

    /// Return iterator over all suitable queues
    pub fn filter_queue<T>(&self, f: T) -> impl Iterator<Item = &QueueFamilyDescription>
    where
        T: Fn(&QueueFamilyDescription) -> bool,
    {
        self.queues().filter(move |x| f(x))
    }

    /// Return first suitable queue or None
    pub fn find_first_queue<T>(&self, f: T) -> Option<&QueueFamilyDescription>
    where
        T: Fn(&QueueFamilyDescription) -> bool,
    {
        self.queues().find(move |x| f(x))
    }

    /// Return iterator over all suitable memory heaps
    pub fn filter_memory<T>(&self, f: T) -> impl Iterator<Item = &MemoryDescription>
    where
        T: Fn(&MemoryDescription) -> bool,
    {
        self.memory().filter(move |x| f(x))
    }

    /// Return first suitable memory or None
    pub fn find_first_memory<T>(&self, f: T) -> Option<&MemoryDescription>
    where
        T: Fn(&MemoryDescription) -> bool,
    {
        self.memory().find(move |x| f(x))
    }
}

// Call unwrap to supress warnings
impl fmt::Display for HWDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "*****************************\n\
            Device: {}\n\
            Device type: {:#?}\n\
            Device id:   {}\n\
            Vendor id:   {}\n\n\
            Supported API information:\n\
            Version major: {}\n\
            Version minor: {}\n\
            Version patch: {}\n",
            self.name(),
            self.device_type(),
            self.hw_id(),
            self.vendor_id(),
            self.version_major(),
            self.version_minor(),
            self.version_patch()
        )
        .unwrap();

        write!(
            f,
            "*****************************\n\
            Features\n\
            *****************************\n\
            {:#?}\n\
            *****************************\n",
            self.i_features
        )
        .unwrap();

        write!(
            f,
            "*****************************\n\
            Queue family information\n\
            *****************************\n"
        )
        .unwrap();

        for (i, queue) in self.i_queues.iter().enumerate() {
            write!(
                f,
                "Queue family number {}\n\
                -----------------------------\n\
                {}\
                -----------------------------\n",
                i, queue
            )
            .unwrap();
        }

        write!(
            f,
            "*****************************\n\
            Memory information\n\
            *****************************\n"
        )
        .unwrap();

        for (i, info) in self.i_heap_info.iter().enumerate() {
            write!(
                f,
                "Memory type {}\n\
                -----------------------------\n\
                {}\
                -----------------------------\n",
                i, info
            )
            .unwrap();
        }

        write!(
            f,
            "*****************************\n\
            Limits\n"
        )
        .unwrap();

        write!(
            f,
            "*****************************\n\
            Min uniform buffer offset: {}\n\
            Min storage buffer offset: {}\n\
            Memory alignment: {}\n",
            self.ubo_offset(),
            self.storage_offset(),
            self.memory_alignment()
        )
        .unwrap();

        Ok(())
    }
}

pub struct Description(Vec<HWDevice>);

impl Description {
    /// Try to retrieve information about hardware
    ///
    /// Pass [`surface`](crate::surface::Surface) to query surface support for each queue family
    ///
    /// If [`None`] was passed no checks will be done and support will be set to default
    ///
    /// See [`is_surface_supported`](crate::hw::QueueFamilyDescription::is_surface_supported)
    pub fn poll(lib: &libvk::Instance, surface: Option<&surface::Surface>) -> Result<Description, HWError> {
        let hw: Vec<vk::PhysicalDevice> = on_error_ret!(
            unsafe { lib.instance().enumerate_physical_devices() },
            HWError::Enumerate
        );

        Ok(Description(
            hw.into_iter().map(|dev| HWDevice::new(lib, dev, surface)).collect(),
        ))
    }

    /// Return iterator over all available hardware devices
    pub fn list(&self) -> impl Iterator<Item = &HWDevice> {
        self.0.iter()
    }

    pub fn filter_hw<T>(&self, selector: T) -> impl Iterator<Item = &HWDevice>
    where
        T: Fn(&HWDevice) -> bool,
    {
        self.list().filter(move |x| selector(x))
    }

    // TODO mb rewrite it with find_map?
    pub fn find_first<T, U, S>(
        &self,
        dev: T,
        queue: U,
        mem: S
    ) -> Option<(&HWDevice, &QueueFamilyDescription, &MemoryDescription)>
    where
        T: Fn(&HWDevice) -> bool,
        U: Fn(&QueueFamilyDescription) -> bool,
        S: Fn(&MemoryDescription) -> bool,
    {
        for hw in self.filter_hw(dev) {
            if let (Some(q), Some(m)) = (hw.find_first_queue(&queue), hw.find_first_memory(&mem)) {
                return Some((hw, q, m));
            }
        }

        None
    }
}

/// Helper function which provides nicer placeholder for filters
pub fn any<T>(_: &T) -> bool {
    true
}
