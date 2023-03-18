//! Buffer which can be used in shaders as uniform buffer
//!
//! Can be used in all stages
use ash::vk;

use crate::{hw, dev, memory, graphics};

fn get_flags(cfg: &memory::MemoryCfg) -> vk::BufferUsageFlags {
    let mut flags = vk::BufferUsageFlags::UNIFORM_BUFFER;

    if cfg.transfer_src {
        flags |= vk::BufferUsageFlags::TRANSFER_SRC;
    }

    if cfg.transfer_dst {
        flags |= vk::BufferUsageFlags::TRANSFER_DST;
    }

    flags
}

/// Represents generic storage buffer
pub struct UniformBuffer(memory::BaseStorage);

impl UniformBuffer {
    /// Note on allocation: if memory is HOST_VISIBLE and is not HOST_COHERENT performs
    /// [map_memory](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkMapMemory.html)
    /// and
    /// [flush](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkFlushMappedMemoryRanges.html)
    /// which may result in [error](memory::MemoryError::MapAccess)
    pub fn allocate(
        device: &dev::Device,
        memory: &hw::MemoryDescription,
        cfg: &memory::MemoryCfg
    ) -> Result<UniformBuffer, memory::MemoryError> {
        match memory::BaseStorage::new(device, memory, &cfg, get_flags(cfg)) {
            Ok(val) => Ok(UniformBuffer(val)),
            Err(e) => Err(e)
        }
    }

    /// Tries to find first suitable memory
    pub fn find_memory<'a, T>(
        device: &'a dev::Device,
        f: T,
        cfg: &'a memory::MemoryCfg
    ) -> Option<&'a hw::MemoryDescription>
    where
        T: Fn(&hw::MemoryDescription) -> bool
    {
        device.hw().find_first_memory(move |m| f(m) && memory::is_compatible(device, m, cfg, get_flags(cfg)))
    }

    /// Return iterator over memories filtered by `f` and compatibility with `cfg`
    pub fn filter_memory<'a, T>(
        device: &'a dev::Device,
        f: T,
        cfg: &'a memory::MemoryCfg
    ) -> impl Iterator<Item = &'a hw::MemoryDescription>
    where
        T: Fn(&hw::MemoryDescription) -> bool
    {
        device.hw().filter_memory(move |m| f(m) && memory::is_compatible(device, m, cfg, get_flags(cfg)))
    }

    /// Performs action on mutable memory
    ///
    /// If memory is not coherent performs
    /// [vkFlushMappedMemoryRanges](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkFlushMappedMemoryRanges.html)
    ///
    /// In other words makes host memory changes available to device
    pub fn write<T, F>(&self, f: &mut F) -> Result<(), memory::MemoryError>
    where
        F: FnMut(&mut [T])
    {
        self.0.write(f)
    }

    /// Return copy of buffer's memory
    ///
    /// If memory is not coherent performs
    /// [vkInvalidateMappedMemoryRanges](https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/vkInvalidateMappedMemoryRanges.html)
    ///
    /// Note: on failure return same error [memory::MemoryError::Flush]
    pub fn read(&self) -> Result<&[u8], memory::MemoryError> {
        self.0.read()
    }

    /// Return size of the buffer in bytes
    pub fn size(&self) -> u64 {
        self.0.size()
    }

    #[doc(hidden)]
    pub fn buffer(&self) -> vk::Buffer {
        self.0.buffer()
    }
}

impl graphics::Resource for UniformBuffer {
    fn resource_type(&self) -> vk::DescriptorType {
        vk::DescriptorType::UNIFORM_BUFFER
    }

    fn count(&self) -> u32 {
        1
    }

    fn layout(&self, stage: graphics::ShaderStage) -> graphics::BindingCfg {
        (self.resource_type(), stage, self.count())
    }

    fn buffer(&self) -> vk::Buffer {
        self.0.buffer()
    }

    fn size(&self) -> u64 {
        self.0.size()
    }
}