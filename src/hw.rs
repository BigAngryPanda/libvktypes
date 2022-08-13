//! Provide information about available hardware
//!
//! Instead of [dev module](crate::dev) `hw` represents hardware level

use ash::vk;

use crate::on_error_ret;
use crate::{libvk, surface};

use std::ffi::CStr;
use std::fmt;

#[derive(Debug)]
pub enum HWError {
    Enumerate,
    SurfaceSupport,
}

/// Represents GPU type
///
#[doc = "See more <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPhysicalDeviceType.html>"]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HWType {
    Unknown,
    Integrated,
    Discrete,
    Virtualized,
    CPU,
}

impl HWType {
    fn new(t: vk::PhysicalDeviceType) -> HWType {
        match t {
            vk::PhysicalDeviceType::INTEGRATED_GPU => HWType::Integrated,
            vk::PhysicalDeviceType::DISCRETE_GPU => HWType::Discrete,
            vk::PhysicalDeviceType::VIRTUAL_GPU => HWType::Virtualized,
            vk::PhysicalDeviceType::CPU => HWType::CPU,
            _ => HWType::Unknown,
        }
    }
}

impl fmt::Display for HWType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                HWType::Unknown => "Unknown",
                HWType::Integrated => "Integrated GPU",
                HWType::Discrete => "Discrete GPU",
                HWType::Virtualized => "Virtual GPU",
                HWType::CPU => "CPU",
            }
        )
    }
}

/// Represent information about single queue family
///
#[doc = "See more <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/html/vkspec.html#VkQueueFlagBits>"]
#[derive(Debug)]
pub struct QueueFamilyDescription {
    i_index: u32,
    i_count: u32,
    i_property: vk::QueueFlags,
}

impl QueueFamilyDescription {
    #[doc(hidden)]
    fn new(property: &vk::QueueFamilyProperties, index: u32) -> QueueFamilyDescription {
        QueueFamilyDescription {
            i_index: index,
            i_count: property.queue_count,
            i_property: property.queue_flags,
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
}

impl fmt::Display for QueueFamilyDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Number of queues:       {}\n\
            Support graphics:       {}\n\
            Support compute:        {}\n\
            Support transfer:       {}\n\
            Support sparce binding: {}\n",
            self.count(),
            if self.is_graphics() { "yes" } else { "no" },
            if self.is_compute()  { "yes" } else { "no" },
            if self.is_transfer() { "yes" } else { "no" },
            if self.is_sparce_binding() {
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
#[derive(Debug, Clone)]
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

pub struct HWDevice {
    i_device: vk::PhysicalDevice,
    i_name: String,
    i_hw_type: HWType,
    i_hw_id: u32,
    i_version: u32,
    i_vendor_id: u32,
    i_queues: Vec<QueueFamilyDescription>,
    i_heap_info: Vec<MemoryDescription>,
}

impl HWDevice {
    fn new(lib: &libvk::Instance, hw: vk::PhysicalDevice) -> HWDevice {
        let properties: vk::PhysicalDeviceProperties =
            unsafe { lib.instance().get_physical_device_properties(hw) };

        let queue_properties: Vec<vk::QueueFamilyProperties> = unsafe {
            lib.instance()
                .get_physical_device_queue_family_properties(hw)
        };

        let memory_desc: Vec<MemoryDescription> = unsafe {
            let mem_props: vk::PhysicalDeviceMemoryProperties =
                lib.instance().get_physical_device_memory_properties(hw);

            mem_props
                .memory_types
                .iter()
                .enumerate()
                .map(|(i, _)| MemoryDescription::new(&mem_props, i))
                .filter(|m| {
                    m.is_local()
                        || m.is_host_visible()
                        || m.is_host_cached()
                        || m.is_host_coherent()
                })
                .collect()
        };

        HWDevice {
            i_device: hw,
            i_name: unsafe {
                CStr::from_ptr(&properties.device_name[0])
                    .to_str()
                    .unwrap()
                    .to_owned()
            },
            i_hw_type: HWType::new(properties.device_type),
            i_hw_id: properties.device_id,
            i_version: properties.api_version,
            i_vendor_id: properties.vendor_id,
            i_queues: queue_properties
                .iter()
                .enumerate()
                .map(|(i, prop)| QueueFamilyDescription::new(prop, i as u32))
                .filter(|q| {
                    q.is_compute() || q.is_graphics() || q.is_transfer() || q.is_sparce_binding()
                })
                .collect(),
            i_heap_info: memory_desc,
        }
    }

    pub fn device(&self) -> vk::PhysicalDevice {
        self.i_device
    }

    /// Device name
    pub fn name(&self) -> &String {
        &self.i_name
    }

    /// Return device type
    pub fn device_type(&self) -> HWType {
        self.i_hw_type
    }

    /// Hardware id
    pub fn hw_id(&self) -> u32 {
        self.i_hw_id
    }

    /// Return packed version
    ///
    #[doc = "About version <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/html/vkspec.html#extendingvulkan-coreversions-versionnumbers>"]
    pub fn version(&self) -> u32 {
        self.i_version
    }

    /// Return API major version number
    ///
    #[doc = "About version <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VK_API_VERSION_MAJOR.html>"]
    pub fn version_major(&self) -> u32 {
        vk::api_version_major(self.i_version)
    }

    /// Return API minor version number
    ///
    #[doc = "About version <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VK_API_VERSION_MINOR.html>"]
    pub fn version_minor(&self) -> u32 {
        vk::api_version_minor(self.i_version)
    }

    /// Return API patch version number
    ///
    #[doc = "About version <https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VK_API_VERSION_PATCH.html>"]
    pub fn version_patch(&self) -> u32 {
        vk::api_version_patch(self.i_version)
    }

    /// Return vendor id
    pub fn vendor_id(&self) -> u32 {
        self.i_vendor_id
    }

    /// Return true if GPU type is `Discrete`
    ///
    /// Otherwise false
    ///
    /// See HWType
    pub fn is_discrete_gpu(&self) -> bool {
        self.i_hw_type == HWType::Discrete
    }

    /// Return true if GPU type is `Integrated`
    ///
    /// Otherwise false
    ///
    /// See HWType
    pub fn is_integrated_gpu(&self) -> bool {
        self.i_hw_type == HWType::Integrated
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

    /// Does selected queue family within hw device supports surface
    pub fn support_surface(
        &self,
        surface: surface::Surface,
        queue_family_index: u32,
    ) -> Result<bool, HWError> {
        match unsafe {
            surface.loader().get_physical_device_surface_support(
                self.device(),
                queue_family_index,
                surface.surface(),
            )
        } {
            Ok(val) => Ok(val),
            Err(_) => Err(HWError::SurfaceSupport),
        }
    }
}

// Call unwrap to supress warnings
impl fmt::Display for HWDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "*****************************\n\
            Device: {}\n\
            Device type: {}\n\
            Device id:   {}\n\
            Vendor id:   {}\n\n\
            Supported API information:\n\
            Version major: {}\n\
            Version minor: {}\n\
            Version patch: {}\n",
            self.i_name,
            self.i_hw_type,
            self.i_hw_id,
            self.i_vendor_id,
            self.version_major(),
            self.version_minor(),
            self.version_patch()
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

        Ok(())
    }
}

pub struct Description(Vec<HWDevice>);

impl Description {
    /// Try to retrieve information about hardware
    pub fn poll(lib: &libvk::Instance) -> Result<Description, HWError> {
        let hw: Vec<vk::PhysicalDevice> = on_error_ret!(
            unsafe { lib.instance().enumerate_physical_devices() },
            HWError::Enumerate
        );

        Ok(Description(
            hw.into_iter().map(|dev| HWDevice::new(lib, dev)).collect(),
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
        mem: S,
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
