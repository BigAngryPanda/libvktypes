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
    data_ptr
};

use std::{
    ptr,
    fmt
};
use std::error::Error;
use std::sync::Arc;

/// Marks that type may be used as binding for shaders
pub trait ShaderBinding {
    fn buffer_info(&self) -> Option<vk::DescriptorBufferInfo>;
    fn image_info(&self) -> Option<vk::DescriptorImageInfo>;
    fn texel_info(&self) -> Option<vk::BufferView>;
}

#[derive(Debug)]
pub enum ResourceError {
    DescriptorPool,
    DescriptorSet,
    DescriptorAllocation
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceError::DescriptorPool => write!(f, "Failed to create descriptor pool (vkCreateDescriptorPool call failed)"),
            ResourceError::DescriptorSet => write!(f, "Failed to create descriptor set layout (vkCreateDescriptorSetLayout call failed)"),
            ResourceError::DescriptorAllocation => write!(f, "Failed to allocate descriptor set (vkDescriptorSetAllocateInfo call failed)"),
        }
    }
}

impl Error for ResourceError { }

/// Specifies how pipeline should treat region of memory
///
#[doc = "Ash documentation about possible values <https://docs.rs/ash/latest/ash/vk/struct.DescriptorType.html>"]
///
#[doc = "Vulkan documentation <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkDescriptorType.html>"]
pub type ResourceType = vk::DescriptorType;

#[derive(Debug, Clone, Copy)]
pub struct BindingCfg {
    pub resource_type: ResourceType,
    pub stage: graphics::ShaderStage,
    pub count: u32,
}

/// Represents information about how many and what type of bindings will be used
///
/// From the creating `PipelineResource` does not contain any information about
/// what exactly memory will be used
///
/// Call [`update`](PipelineResource::update) to write information into `PipelineResource`
#[derive(Debug)]
pub struct PipelineDescriptor {
    i_core: Arc<dev::Core>,
    i_desc_types: Vec<Vec<ResourceType>>,
    i_desc_pool: vk::DescriptorPool,
    i_desc_sets: Vec<vk::DescriptorSet>,
    i_desc_layouts: Vec<vk::DescriptorSetLayout>
}

impl PipelineDescriptor {
    /// Create new `PipelineResource` with fully specified bindings
    pub fn allocate(device: &dev::Device, cfg: &[&[BindingCfg]]) -> Result<PipelineDescriptor, ResourceError> {
        let mut desc_size: Vec<vk::DescriptorPoolSize> = Vec::new();
        let mut desc_types: Vec<Vec<ResourceType>> = Vec::new();

        for &set in cfg {
            let mut set_types: Vec<ResourceType> = Vec::new();

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
            Err(..) => return Err(ResourceError::DescriptorPool),
        };

        let mut sets_layout: Vec<vk::DescriptorSetLayout> = Vec::new();

        for &res in cfg {
            match create_set_layout(device, res) {
                Ok(set) => sets_layout.push(set),
                Err(_) => {
                    clear_sets_layout(device, &sets_layout, desc_pool);
                    return Err(ResourceError::DescriptorSet);
                }
            }
        };

        let sets = on_error!(
            allocate_descriptor_sets(device, &sets_layout, desc_pool),
            {
                clear_sets_layout(device, &sets_layout, desc_pool);
                return Err(ResourceError::DescriptorAllocation);
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
    pub fn with_set(device: &dev::Device, set: &[BindingCfg], count: usize) -> Result<PipelineDescriptor, ResourceError> {
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
    ) -> Result<PipelineDescriptor, ResourceError> {
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

    /// Update all sets
    ///
    /// Each element in `sets` will be bound to the corresponding set
    ///
    /// Example in glsl
    ///
    /// Element sets[X][Y] will be bound to
    ///
    /// ```ignore
    ///     layout(set=X, binding=Y) ...
    /// ```
    ///
    /// Note: order is important
    ///
    /// `layout(set=X, binding=Y) ...` must be before `layout(set=X+1, binding=Y)`
    ///
    /// Otherwise sets[X][Y] will be bound with `layout(set=X+1, binding=Y)`
    ///
    /// If you want to skip any set leave corresponding array empty
    ///
    /// Each array `&[&dyn ShaderBinding]` corresponding to the single binding
    ///
    /// ```ignore
    ///     layout(set=X, binding=Y) ... <binding name>[<count, omit if count == 1>];
    /// ```
    ///
    /// ```len(&[&dyn ShaderBinding]) == count```
    pub fn update(&self, sets: &[&[&[&dyn ShaderBinding]]]) {
        for i in 0..sets.len() {
            self.update_set(sets[i], i);
        }
    }


    /// Update selected set `layout(set=X, binding=...) ...`
    ///
    /// See [`update`] for the detailed docs
    pub fn update_set(&self, set: &[&[&dyn ShaderBinding]], set_index: usize) {
        // info for the whole set
        let mut buffer_info: Vec<Vec<vk::DescriptorBufferInfo>> = Vec::new();
        let mut image_info: Vec<Vec<vk::DescriptorImageInfo>> = Vec::new();
        let mut texel_info: Vec<Vec<vk::BufferView>> = Vec::new();

        for &binding in set {
            // info for each binding
            let infos = binding_infos(binding);

            buffer_info.push(infos.0);
            image_info.push(infos.1);
            texel_info.push(infos.2);
        }

        let write_desc: Vec<vk::WriteDescriptorSet> = set.iter().enumerate().map(
            |(i, _)| vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: ptr::null(),
                dst_set: self.i_desc_sets[set_index],
                dst_binding: i as u32,
                dst_array_element: 0,
                descriptor_count: std::cmp::max(buffer_info.len(), std::cmp::max(image_info.len(), texel_info.len())) as u32,
                descriptor_type: self.i_desc_types[set_index][i],
                p_image_info: data_ptr!(image_info[i]),
                p_buffer_info: data_ptr!(buffer_info[i]),
                p_texel_buffer_view: data_ptr!(texel_info[i])
            }
        ).collect();

        unsafe {
            self.i_core.device().update_descriptor_sets(&write_desc, &[])
        };
    }

    /// Update single binding
    ///
    /// From performance side
    /// if you want to update the whole set better to use [`update_set`]
    pub fn update_binding(&self, binding: &[&dyn ShaderBinding], set_index: usize, binding_index: usize) {
        let infos = binding_infos(binding);

        let write_info = vk::WriteDescriptorSet {
            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
            p_next: ptr::null(),
            dst_set: self.i_desc_sets[set_index],
            dst_binding: binding_index as u32,
            dst_array_element: 0,
            descriptor_count: std::cmp::max(infos.0.len(), std::cmp::max(infos.1.len(), infos.2.len())) as u32,
            descriptor_type: self.i_desc_types[set_index][binding_index],
            p_image_info: data_ptr!(infos.1),
            p_buffer_info: data_ptr!(infos.0),
            p_texel_buffer_view: data_ptr!(infos.2)
        };


        unsafe {
            self.i_core.device().update_descriptor_sets(&[write_info], &[])
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
            p_immutable_samplers: ptr::null()
        }
    ).collect();

    let desc_layout_info = vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: bindings.len() as u32,
        p_bindings: bindings.as_ptr(),
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
        p_set_layouts: sets.as_ptr()
    };

    unsafe {
        device.device().allocate_descriptor_sets(&alloc_info)
    }
}

fn binding_infos(binding: &[&dyn ShaderBinding])
-> (Vec<vk::DescriptorBufferInfo>, Vec<vk::DescriptorImageInfo>, Vec<vk::BufferView>) {
    let mut buffer_info: Vec<vk::DescriptorBufferInfo> = Vec::new();
    let mut image_info: Vec<vk::DescriptorImageInfo> = Vec::new();
    let mut texel_info: Vec<vk::BufferView> = Vec::new();

    for &elem in binding {
        if let Some(info) = elem.buffer_info() {
            buffer_info.push(info);
        }

        if let Some(info) = elem.image_info() {
            image_info.push(info);
        }

        if let Some(info) = elem.texel_info() {
            texel_info.push(info);
        }
    }

    (buffer_info, image_info, texel_info)
}
