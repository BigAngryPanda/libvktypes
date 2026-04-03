use ash::vk;

use ash::prelude::VkResult;

use crate::{
    dev,
    memory,
    graphics,
    pipeline
};

use crate::{
    on_error,
    data_ptr
};

use std::sync::Arc;
use std::marker::PhantomData;

pub type BufferInfo = vk::DescriptorBufferInfo;
pub type ImageInfo  = vk::DescriptorImageInfo;

/// Represents writing information for single layout entry
/// ```layout(set, binding) ... data[N]; ```
#[derive(Debug)]
pub struct WriteInfoEntry<T> {
    set: u32,
    binding: u32,
    starting_array_element: u32,
    desc_type: pipeline::DescriptorType,
    data: Vec<T>
}

impl<T> WriteInfoEntry<T> {
    fn new(set: u32, binding: u32, desc: pipeline::DescriptorType) -> WriteInfoEntry<T> {
        WriteInfoEntry {
            set,
            binding,
            starting_array_element: 0,
            desc_type: desc,
            data: Vec::new(),
        }
    }

    pub fn starting_element(&mut self, idx: u32) -> &mut Self {
        self.starting_array_element = idx;

        self
    }
}

impl WriteInfoEntry<BufferInfo> {
    /// Add buffer for writing
    ///
    /// Call multiple times for array writing
    pub fn element<T: memory::BufferView>(&mut self, view: T) -> &mut Self {
        self.element_with_params(view, 0, vk::WHOLE_SIZE)
    }

    /// See [`value`](Self::element)
    pub fn element_with_params<T: memory::BufferView>(&mut self,
        view: T,
        offset: u64,
        range: u64
    ) -> &mut Self {
        self.data.push(vk::DescriptorBufferInfo {
                    buffer: memory::get_buffer(view),
                    offset: offset,
                    range: range,
                });

        self
    }
}

impl WriteInfoEntry<ImageInfo> {
    pub fn element<U: memory::ImageView>(&mut self,
        view: U,
        sampler: &graphics::Sampler,
        layout: memory::ImageLayout
    ) -> &mut Self {
        self.data.push(vk::DescriptorImageInfo {
                    sampler: sampler.sampler(),
                    image_view: memory::get_image_view(view),
                    image_layout: layout,
                });

        self
    }
}

/// Allocates memory in the heap
///
/// Better to prepare info in advance
#[derive(Debug)]
pub struct WriteInfo {
    pub(crate) buffers: Vec<WriteInfoEntry<vk::DescriptorBufferInfo>>,
    pub(crate) images:  Vec<WriteInfoEntry<vk::DescriptorImageInfo>>
}

impl WriteInfo {
    pub fn new() -> WriteInfo {
        WriteInfo {
            buffers: Vec::new(),
            images: Vec::new()
        }
    }

    /// Allocate new entry for buffer binding
    pub fn buffer(&mut self, set: u32, binding: u32, desc: pipeline::DescriptorType) ->
        &mut WriteInfoEntry<BufferInfo>
    {
        self.buffers.push(WriteInfoEntry::new(set, binding, desc));

        self.buffers.last_mut().unwrap()
    }

    /// Allocate new entry for image binding
    pub fn image(&mut self, set: u32, binding: u32, desc: pipeline::DescriptorType) ->
        &mut WriteInfoEntry<ImageInfo>
    {
        self.images.push(WriteInfoEntry::new(set, binding, desc));

        self.images.last_mut().unwrap()
    }
}

/// Contains information about what buffer or image will be passed to the pipeline
#[derive(Debug)]
pub struct PipelineBindings {
    i_core: Arc<dev::Core>,
    i_desc_pool: vk::DescriptorPool,
    i_desc_sets: Vec<vk::DescriptorSet>,
}

impl PipelineBindings {
    pub fn new(device: &dev::Device, layout: &pipeline::PipelineLayout) ->
        Result<PipelineBindings, pipeline::BindingError>
    {
        let mut desc_size: Vec<vk::DescriptorPoolSize> = Vec::new();

        for set in layout.bindings() {
            for binding in set {
                desc_size.push(vk::DescriptorPoolSize {
                    ty: binding.resource_type,
                    descriptor_count: binding.count
                });
            }
        }

        let desc_pool = match create_descriptor_pool(device, &desc_size) {
            Ok(val) => if val == vk::DescriptorPool::null() { return Ok(Self::empty(device)) } else { val },
            Err(..) => return Err(pipeline::BindingError::DescriptorPool),
        };

        let sets = on_error!(
            allocate_descriptor_sets(device, layout.sets_layouts(), desc_pool),
            {
                unsafe {
                    device
                    .device()
                    .destroy_descriptor_pool(desc_pool, device.allocator());
                }
                return Err(pipeline::BindingError::DescriptorAllocation);
            }
        );

        Ok(PipelineBindings {
            i_core: device.core().clone(),
            i_desc_pool: desc_pool,
            i_desc_sets: sets
        })
    }

    pub fn empty(device: &dev::Device) -> PipelineBindings {
        PipelineBindings {
            i_core: device.core().clone(),
            i_desc_pool: vk::DescriptorPool::null(),
            i_desc_sets: Vec::new()
        }
    }

    pub fn write(&self, write_info: &WriteInfo) {
        let mut write_desc: Vec<vk::WriteDescriptorSet> =
            Vec::with_capacity(write_info.buffers.len() + write_info.images.len());

        for info in &write_info.buffers {
            write_desc.push(vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: std::ptr::null(),
                dst_set: self.i_desc_sets[info.set as usize],
                dst_binding: info.binding,
                dst_array_element: info.starting_array_element,
                descriptor_count: info.data.len() as u32,
                descriptor_type: info.desc_type,
                p_image_info: std::ptr::null(),
                p_buffer_info: data_ptr!(info.data),
                p_texel_buffer_view: std::ptr::null(),
                _marker: PhantomData,
            });
        }

        for info in &write_info.images {
            write_desc.push(vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: std::ptr::null(),
                dst_set: self.i_desc_sets[info.set as usize],
                dst_binding: info.binding,
                dst_array_element: info.starting_array_element,
                descriptor_count: info.data.len() as u32,
                descriptor_type: info.desc_type,
                p_image_info: data_ptr!(info.data),
                p_buffer_info: std::ptr::null(),
                p_texel_buffer_view: std::ptr::null(),
                _marker: PhantomData,
            });
        }

        unsafe {
            self.i_core.device().update_descriptor_sets(&write_desc, &[])
        };
    }

    pub(crate) fn descriptors(&self) -> &[vk::DescriptorSet] {
        &self.i_desc_sets
    }
}

impl Drop for PipelineBindings {
    fn drop(&mut self) {
        let device = self.i_core.device();
        let alloc  = self.i_core.allocator();

        unsafe {
            if self.i_desc_pool != vk::DescriptorPool::null() {
                device
                .destroy_descriptor_pool(self.i_desc_pool, alloc);
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
        p_next: std::ptr::null(),
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

fn allocate_descriptor_sets(
    device: &dev::Device,
    sets: &Vec<vk::DescriptorSetLayout>,
    pool: vk::DescriptorPool
) -> VkResult<Vec<vk::DescriptorSet>> {
    let alloc_info = vk::DescriptorSetAllocateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
        p_next: std::ptr::null(),
        descriptor_pool: pool,
        descriptor_set_count: sets.len() as u32,
        p_set_layouts: sets.as_ptr(),
        _marker: PhantomData,
    };

    unsafe {
        device.device().allocate_descriptor_sets(&alloc_info)
    }
}
