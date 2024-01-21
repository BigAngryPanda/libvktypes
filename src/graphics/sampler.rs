//! Sampler struct for texturing

use ash::vk;

use crate::{
    dev,
    graphics,
    on_error_ret
};

use std::{
    ptr,
    fmt
};
use std::error::Error;
use std::sync::Arc;

/// Specify behavior of sampling with texture coordinates outside an image
///
#[doc = "Ash documentation about possible values <https://docs.rs/ash/latest/ash/vk/struct.SamplerAddressMode.html>"]
///
#[doc = "Vulkan documentation <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkSamplerAddressMode.html>"]
pub type SamplerAddressMode = vk::SamplerAddressMode;

/// Specify mipmap mode used for texture lookups
///
#[doc = "Ash documentation about possible values <https://docs.rs/ash/latest/ash/vk/struct.SamplerMipmapMode.html>"]
///
#[doc = "Vulkan documentation <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkSamplerMipmapMode.html>"]
pub type SamplerMipmapMode = vk::SamplerMipmapMode;

/// Specify filters used for texture lookups
///
#[doc = "Ash documentation about possible values <https://docs.rs/ash/latest/ash/vk/struct.Filter.html>"]
///
#[doc = "Vulkan documentation <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkFilter.html>"]
pub type SamplerFilter = vk::Filter;

/// Specify border color used for texture lookups
///
#[doc = "Ash documentation about possible values <https://docs.rs/ash/latest/ash/vk/struct.BorderColor.html>"]
///
#[doc = "Vulkan documentation <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkBorderColor.html>"]
pub type BorderColor = vk::BorderColor;

#[derive(Debug)]
pub enum SamplerError {
    Creation
}

impl fmt::Display for SamplerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vkCreateSampler call failed")
    }
}

impl Error for SamplerError {}

/// Sampler creation configuration
///
/// For fields description see
/// [`VkSamplerCreateInfo`](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkSamplerCreateInfo.html)
pub struct SamplerCfg {
    pub mipmap_mode: SamplerMipmapMode,
    pub address_mode_u: SamplerAddressMode,
    pub address_mode_v: SamplerAddressMode,
    pub address_mode_w: SamplerAddressMode,
    pub mag_filter: SamplerFilter,
    pub min_filter: SamplerFilter,
    pub mip_lod_bias: f32,
    pub anisotropy_enable: bool,
    pub max_anisotropy: f32,
    pub compare_enable: bool,
    pub compare_op: graphics::CompareOp,
    pub min_lod: f32,
    pub max_lod: f32,
    pub border_color: BorderColor,
    pub unnormalized_coordinates: bool,
}

impl Default for SamplerCfg {
    /// Default values are:
    /// ```ignore
    /// mipmap_mode: LINEAR
    /// address_mode_u: REPEAT
    /// address_mode_v: REPEAT
    /// address_mode_w: REPEAT
    /// mag_filter: LINEAR
    /// min_filter: LINEAR
    /// mip_lod_bias: 0.0
    /// anisotropy_enable: false
    /// max_anisotropy: 0.0
    /// compare_enable: false
    /// compare_op: ALWAYS
    /// min_lod: 0.0
    /// max_lod: 0.0
    /// border_color: INT_OPAQUE_BLACK
    /// unnormalized_coordinates: false
    /// ```
    fn default() -> Self {
        SamplerCfg {
            mipmap_mode: SamplerMipmapMode::LINEAR,
            address_mode_u: SamplerAddressMode::REPEAT,
            address_mode_v: SamplerAddressMode::REPEAT,
            address_mode_w: SamplerAddressMode::REPEAT,
            mag_filter: SamplerFilter::LINEAR,
            min_filter: SamplerFilter::LINEAR,
            mip_lod_bias: 0.0,
            anisotropy_enable: false,
            max_anisotropy: 0.0,
            compare_enable: false,
            compare_op: graphics::CompareOp::ALWAYS,
            min_lod: 0.0,
            max_lod: 0.0,
            border_color: BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: false,
        }
    }
}

/// Sampler struct itself
#[derive(Debug)]
pub struct Sampler {
    i_core: Arc<dev::Core>,
    i_sampler: vk::Sampler,
}

impl Sampler {
    pub fn new(device: &dev::Device, cfg: &SamplerCfg) -> Result<Sampler, SamplerError> {
        let info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SamplerCreateFlags::empty(),
            mag_filter: cfg.mag_filter,
            min_filter: cfg.min_filter,
            mipmap_mode: cfg.mipmap_mode,
            address_mode_u: cfg.address_mode_u,
            address_mode_v: cfg.address_mode_v,
            address_mode_w: cfg.address_mode_w,
            mip_lod_bias: cfg.mip_lod_bias,
            anisotropy_enable: cfg.anisotropy_enable as u32,
            max_anisotropy: cfg.max_anisotropy,
            compare_enable: cfg.compare_enable as u32,
            compare_op: cfg.compare_op,
            min_lod: cfg.min_lod,
            max_lod: cfg.max_lod,
            border_color: cfg.border_color,
            unnormalized_coordinates: cfg.unnormalized_coordinates as u32,
        };

        let sampler = unsafe {
            on_error_ret!(device.device().create_sampler(&info, device.allocator()), SamplerError::Creation)
        };

        Ok(
            Sampler {
                i_core: device.core().clone(),
                i_sampler: sampler,
            }
        )
    }

    pub(crate) fn sampler(&self) -> vk::Sampler {
        self.i_sampler
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            self.i_core.device().destroy_sampler(self.i_sampler, self.i_core.allocator());
        }
    }
}
