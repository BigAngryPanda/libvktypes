//! Connection between shaders, pipeline and memory
//!
//! Note: vertex buffers, images are used for drawing but sort of separate entities with different workflow
//! (despite they are used by shaders one way or another)

use ash::vk;
use ash::prelude::VkResult;

use crate::{
    dev,
    graphics,
    on_error,
    data_ptr,
    memory
};

use std::{
    ptr,
    fmt
};
use std::error::Error;
use std::sync::Arc;
use std::marker::PhantomData;

/// Represents [Vulkan struct](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkDescriptorBufferInfo.html)
///
/// For `offset` and `range` look in the link above
#[derive(Debug, Clone, Copy)]
pub struct BufferBinding<T: memory::BufferView> {
    pub view: T,
    pub offset: u64,
    pub range: u64,
}

impl<T: memory::BufferView> BufferBinding<T> {
    /// Use this method to create BufferBinding with default params
    ///
    /// It is suitable if you don't have dynamic buffers
    pub fn new(view: T) -> BufferBinding<T> {
        BufferBinding {
            view,
            offset: 0,
            range: vk::WHOLE_SIZE,
        }
    }

    /// Suitable for dynamic buffers
    pub fn with_params(view: T, offset: u64, range: u64) -> BufferBinding<T> {
        BufferBinding {
            view,
            offset,
            range,
        }
    }
}

/// Information for binding textures
#[derive(Debug)]
pub struct SamplerBinding<U: memory::ImageView> {
    pub sampler: graphics::Sampler,
    pub view: U,
    pub layout: memory::ImageLayout,
}

impl<U: memory::ImageView> SamplerBinding<U> {
    pub fn new(
        sampler: graphics::Sampler,
        view: U,
        layout: memory::ImageLayout
    ) -> SamplerBinding<U> {
        SamplerBinding {
            sampler,
            view,
            layout,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ShaderBinding<'a, T: memory::BufferView, U: memory::ImageView> {
    Buffers(&'a [BufferBinding<T>]),
    Samplers(&'a [SamplerBinding<U>]),
}

impl<'a, T: memory::BufferView, U: memory::ImageView> ShaderBinding<'a, T, U> {
    pub fn len(&self) -> u32 {
        match self {
            Self::Buffers(val)  => val.len() as u32,
            Self::Samplers(val) => val.len() as u32,
        }
    }
}

#[derive(Debug)]
pub enum PipelineDescriptorError {
    DescriptorPool,
    DescriptorSet,
    DescriptorAllocation
}

impl fmt::Display for PipelineDescriptorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipelineDescriptorError::DescriptorPool => write!(f, "Failed to create descriptor pool (vkCreateDescriptorPool call failed)"),
            PipelineDescriptorError::DescriptorSet => write!(f, "Failed to create descriptor set layout (vkCreateDescriptorSetLayout call failed)"),
            PipelineDescriptorError::DescriptorAllocation => write!(f, "Failed to allocate descriptor set (vkDescriptorSetAllocateInfo call failed)"),
        }
    }
}

impl Error for PipelineDescriptorError { }

/// Specifies how pipeline should treat region of memory
///
#[doc = "Ash documentation about possible values <https://docs.rs/ash/latest/ash/vk/struct.DescriptorType.html>"]
///
#[doc = "Vulkan documentation <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkDescriptorType.html>"]
pub type DescriptorType = vk::DescriptorType;

/// Information about what Descriptor to write
#[derive(Debug, Clone, Copy)]
pub struct UpdateInfo<'a, T: memory::BufferView, U: memory::ImageView> {
    /// Which set X in layout(set=X, ...) to update
    pub set: usize,
    /// Which binding Y in layout(set=X, binding=Y) to update
    pub binding: u32,
    /// Starting array element in binding `layout(...) ... data[N]`
    ///
    /// `starting_array_element` < N
    pub starting_array_element: u32,
    /// What buffer or image to use
    ///
    /// Note: resource must match corresponding [`DescriptorType`](BindingCfg::resource_type)
    ///
    /// Read more in [spec](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkDescriptorType.html)
    pub resources: ShaderBinding<'a, T, U>,
}

/// Specify what binding to allocate
#[derive(Debug, Clone, Copy)]
pub struct BindingCfg {
    pub resource_type: DescriptorType,
    pub stage: graphics::ShaderStage,
    pub count: u32,
}

/// Represents information about how many and what type of bindings will be used
///
/// From the creating `PipelineDescriptor` does not contain any information about
/// what exactly memory will be used
///
/// Call [`update`](PipelineDescriptor::update) to write information into `PipelineDescriptor`
#[derive(Debug)]
pub struct PipelineDescriptor {
    i_core: Arc<dev::Core>,
    i_desc_types: Vec<Vec<DescriptorType>>,
    i_desc_pool: vk::DescriptorPool,
    i_desc_sets: Vec<vk::DescriptorSet>,
    i_desc_layouts: Vec<vk::DescriptorSetLayout>
}

impl PipelineDescriptor {
    /// Create new `PipelineResource` with fully specified bindings
    ///
    /// `PipelineDescriptor` supports `cfg.len()` sets
    ///
    /// Each set supports `cfg[i].len()` bindings
    ///
    /// Each binding within set supports `BindingCfg::count` array elements
    ///
    /// For binding `(set=i, binding=j) cfg[i][j]` will be used
    pub fn allocate(device: &dev::Device, cfg: &[&[BindingCfg]]) -> Result<PipelineDescriptor, PipelineDescriptorError> {
        let mut desc_size: Vec<vk::DescriptorPoolSize> = Vec::new();
        let mut desc_types: Vec<Vec<DescriptorType>> = Vec::new();

        for &set in cfg {
            let mut set_types: Vec<DescriptorType> = Vec::new();

            for binding in set {
                desc_size.push(vk::DescriptorPoolSize {
                    ty: binding.resource_type,
                    descriptor_count: binding.count
                });

                set_types.push(binding.resource_type);
            }

            desc_types.push(set_types);
        }

        let desc_pool = match create_descriptor_pool(device, &desc_size) {
            Ok(val) => if val == vk::DescriptorPool::null() { return Ok(PipelineDescriptor::empty(device)) } else { val },
            Err(..) => return Err(PipelineDescriptorError::DescriptorPool),
        };

        let mut sets_layout: Vec<vk::DescriptorSetLayout> = Vec::new();

        for &res in cfg {
            match create_set_layout(device, res) {
                Ok(set) => sets_layout.push(set),
                Err(_) => {
                    clear_sets_layout(device, &sets_layout, desc_pool);
                    return Err(PipelineDescriptorError::DescriptorSet);
                }
            }
        };

        let sets = on_error!(
            allocate_descriptor_sets(device, &sets_layout, desc_pool),
            {
                clear_sets_layout(device, &sets_layout, desc_pool);
                return Err(PipelineDescriptorError::DescriptorAllocation);
            }
        );

        Ok(PipelineDescriptor {
            i_core: device.core().clone(),
            i_desc_types: desc_types,
            i_desc_pool: desc_pool,
            i_desc_sets: sets,
            i_desc_layouts: sets_layout
        })
    }

    /// Create new `PipelineResource` with the same set type but (possibly) distinct bindings repeated `count` times
    ///
    /// Example:
    /// ```ignore
    /// layout(set=0, binding=0) <type 1> {...}
    /// layout(set=0, binding=1) <type 2 (possibly type 1 == type 2)> {...}
    /// ...
    /// layout(set=X, binding=0) <type 1> {...}
    /// layout(set=X, binding=1) <type 2> {...}
    ///
    /// // X == count
    /// ```
    pub fn with_set(device: &dev::Device, set: &[BindingCfg], count: usize) -> Result<PipelineDescriptor, PipelineDescriptorError> {
        let cfg = vec![set; count];

        PipelineDescriptor::allocate(device, &cfg)
    }

    /// Create new `PipelineResource` with the same binding in each set
    ///
    /// Example:
    /// ```ignore
    /// layout(set=0, binding=0) <type 1> {...}
    /// ...
    /// layout(set=0, binding=Y) <type 1> {...}
    /// ...
    /// layout(set=X, binding=0) <type 1> {...}
    /// ...
    /// layout(set=X, binding=Y) <type 1> {...}
    ///
    /// // X == sets_num
    /// // Y == count
    /// ```
    pub fn with_sets(
        device: &dev::Device,
        cfg: BindingCfg,
        sets_num: usize,
        count: usize
    ) -> Result<PipelineDescriptor, PipelineDescriptorError> {
        let cfg = vec![cfg; sets_num];

        PipelineDescriptor::with_set(device, &cfg, count)
    }

    /// Create new `PipelineResource` with no bindings
    pub fn empty(device: &dev::Device) -> PipelineDescriptor {
        PipelineDescriptor {
            i_core: device.core().clone(),
            i_desc_types: Vec::new(),
            i_desc_pool: vk::DescriptorPool::null(),
            i_desc_sets: Vec::new(),
            i_desc_layouts: Vec::new()
        }
    }

    /// Does resource contain any bindings
    pub fn is_empty(&self) -> bool {
        self.i_desc_pool == vk::DescriptorPool::null()
    }

    /// Update selected elements in bindings
    ///
    /// `UpdateInfo::set` `UpdateInfo::binding` `UpdateInfo::starting_array_element`
    /// must be within supported range
    ///
    /// About supported ranges see [`PipelineDescriptor::allocate`]
    pub fn update<T: memory::BufferView, U: memory::ImageView>(&self, update_info: &[UpdateInfo<T, U>]) {
        let mut buffer_info: Vec<Vec<vk::DescriptorBufferInfo>> = Vec::new();
        let mut image_info: Vec<Vec<vk::DescriptorImageInfo>> = Vec::new();

        for info in update_info {
            buffer_info.push(create_buffer_info(info.resources));
            image_info.push(create_image_info(info.resources));
        }

        let write_desc: Vec<vk::WriteDescriptorSet> = update_info.iter().enumerate().map(
            |(i, info)| vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: ptr::null(),
                dst_set: self.i_desc_sets[info.set],
                dst_binding: info.binding,
                dst_array_element: info.starting_array_element,
                descriptor_count: info.resources.len(),
                descriptor_type: self.i_desc_types[info.set][info.binding as usize],
                p_image_info: data_ptr!(image_info[i]),
                p_buffer_info: data_ptr!(buffer_info[i]),
                p_texel_buffer_view: ptr::null(),
                _marker: PhantomData,
            }
        ).collect();

        unsafe {
            self.i_core.device().update_descriptor_sets(&write_desc, &[])
        };
    }

    pub(crate) fn descriptor_sets(&self) -> &[vk::DescriptorSet] {
        &self.i_desc_sets
    }

    pub(crate) fn descriptor_layouts(&self) -> &[vk::DescriptorSetLayout] {
        &self.i_desc_layouts
    }
}

impl Drop for PipelineDescriptor {
    fn drop(&mut self) {
        unsafe {
            if self.i_desc_pool != vk::DescriptorPool::null() {
                self
                .i_core
                .device()
                .destroy_descriptor_pool(self.i_desc_pool, self.i_core.allocator());
                for &set in &self.i_desc_layouts {
                    self
                    .i_core
                    .device()
                    .destroy_descriptor_set_layout(set, self.i_core.allocator());
                }
            }
        }
    }
}

fn create_descriptor_pool(
    device: &dev::Device,
    desc_size: &Vec<vk::DescriptorPoolSize>
) -> VkResult<vk::DescriptorPool> {
    let desc_info = vk::DescriptorPoolCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: desc_size.len() as u32,
        pool_size_count: desc_size.len() as u32,
        p_pool_sizes: desc_size.as_ptr(),
        _marker: PhantomData,
    };

    unsafe {
        device.device().create_descriptor_pool(&desc_info, device.allocator())
    }
}

fn create_set_layout(
    device: &dev::Device,
    resources: &[BindingCfg]
) -> VkResult<vk::DescriptorSetLayout> {
    let bindings: Vec<vk::DescriptorSetLayoutBinding> = resources.iter().enumerate().map(
        |(i, binding)| vk::DescriptorSetLayoutBinding {
            binding: i as u32,
            descriptor_type: binding.resource_type,
            descriptor_count: binding.count,
            stage_flags: binding.stage,
            p_immutable_samplers: ptr::null(),
            _marker: PhantomData,
        }
    ).collect();

    let desc_layout_info = vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: bindings.len() as u32,
        p_bindings: bindings.as_ptr(),
        _marker: PhantomData,
    };

    unsafe {
        device.device().create_descriptor_set_layout(&desc_layout_info, device.allocator())
    }
}

fn clear_sets_layout(
    device: &dev::Device,
    sets: &Vec<vk::DescriptorSetLayout>,
    pool: vk::DescriptorPool)
{
    if pool == vk::DescriptorPool::null() {
        return;
    }

    unsafe {
        device
        .device()
        .destroy_descriptor_pool(pool, device.allocator());

        for &set in sets {
            device
            .device()
            .destroy_descriptor_set_layout(set, device.allocator());
        }
    }
}

fn allocate_descriptor_sets(
    device: &dev::Device,
    sets: &Vec<vk::DescriptorSetLayout>,
    pool: vk::DescriptorPool
) -> VkResult<Vec<vk::DescriptorSet>> {
    let alloc_info = vk::DescriptorSetAllocateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
        p_next: ptr::null(),
        descriptor_pool: pool,
        descriptor_set_count: sets.len() as u32,
        p_set_layouts: sets.as_ptr(),
        _marker: PhantomData,
    };

    unsafe {
        device.device().allocate_descriptor_sets(&alloc_info)
    }
}

fn create_image_info<T: memory::BufferView, U: memory::ImageView>(
    bindings: ShaderBinding<T, U>
) -> Vec<vk::DescriptorImageInfo> {
    match bindings {
        ShaderBinding::Buffers(_) => {
            Vec::new()
        }
        ShaderBinding::Samplers(samplers) => {
            descriptor_image_info(&samplers)
        }
    }
}

fn descriptor_image_info<U: memory::ImageView>(
    samplers: &[SamplerBinding<U>]
) -> Vec<vk::DescriptorImageInfo> {
    samplers
    .iter()
    .map(|binding| {
        vk::DescriptorImageInfo {
            sampler: binding.sampler.sampler(),
            image_view: memory::get_image_view(binding.view),
            image_layout: binding.layout,
        }
    }).collect()
}

fn create_buffer_info<T: memory::BufferView, U: memory::ImageView>(
    bindings: ShaderBinding<T, U>
) -> Vec<vk::DescriptorBufferInfo> {
    match bindings {
        ShaderBinding::Buffers(buffers) => {
            descriptor_buffer_info(&buffers)
        }
        ShaderBinding::Samplers(_) => {
            Vec::new()
        }
    }
}

fn descriptor_buffer_info<T: memory::BufferView>(
    buffers: &[BufferBinding<T>]
) -> Vec<vk::DescriptorBufferInfo>  {
    buffers
    .iter()
    .map(|binding| {
        vk::DescriptorBufferInfo {
            buffer: memory::get_buffer(binding.view),
            offset: binding.offset,
            range: binding.range,
        }
    }).collect()
}