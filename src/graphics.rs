//! Graphics pipeline and render pass
//!
//! # RenderPass
//! [`RenderPass`] represents context within graphics pipeline is executed
//!
//! It is defined by 3 components:
//! 1) [subpasses](SubpassInfo)
//! 2) [synchronization between subpasses](SubpassSync)
//! 3) [attachments](AttachmentInfo) which defines what for *all* images are used for

use ash::vk;

use crate::{
    dev,
    surface,
    data_ptr,
    on_error_ret
};

use std::ptr;
use std::fmt;
use std::error::Error;
use std::convert::Into;

/// Specify how contents of an attachment are treated at the beginning of a subpass
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.AttachmentLoadOp.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkAttachmentLoadOp.html>"]
pub type AttachmentLoadOp = vk::AttachmentLoadOp;

/// Specify how contents of an attachment are treated at the end of a subpass
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.AttachmentStoreOp.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkAttachmentStoreOp.html>"]
pub type AttachmentStoreOp = vk::AttachmentStoreOp;

/// Layout of image and image subresources
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.ImageLayout.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkImageLayout.html>"]
pub type ImageLayout = vk::ImageLayout;

/// Structure specifying an attachment description
#[derive(Debug)]
pub struct AttachmentInfo {
    pub format: surface::ImageFormat,
    pub load_op: AttachmentLoadOp,
    pub store_op: AttachmentStoreOp,
    pub stencil_load_op: AttachmentLoadOp,
    pub stencil_store_op: AttachmentStoreOp,
    pub initial_layout: ImageLayout,
    pub final_layout: ImageLayout,
}

impl Default for AttachmentInfo {
    fn default() -> Self {
        AttachmentInfo {
            format: surface::ImageFormat::UNDEFINED,
            load_op: AttachmentLoadOp::DONT_CARE,
            store_op: AttachmentStoreOp::DONT_CARE,
            stencil_load_op: AttachmentLoadOp::DONT_CARE,
            stencil_store_op: AttachmentStoreOp::DONT_CARE,
            initial_layout: ImageLayout::PRESENT_SRC_KHR,
            final_layout: ImageLayout::PRESENT_SRC_KHR,
        }
    }
}

#[doc(hidden)]
impl From<&AttachmentInfo> for vk::AttachmentDescription {
    fn from(info: &AttachmentInfo) -> vk::AttachmentDescription {
        vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: info.format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: info.load_op,
            store_op: info.store_op,
            stencil_load_op: info.stencil_load_op,
            stencil_store_op: info.stencil_store_op,
            initial_layout: info.initial_layout,
            final_layout: info.final_layout,
        }
    }
}

/// Pipeline stages
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.PipelineStageFlags.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkPipelineStageFlagBits.html>"]
pub type PipelineStage = vk::PipelineStageFlags;

/// Bitmask specifying memory access types that will participate in a memory dependency
///
#[doc = "Values: <https://docs.rs/ash/latest/ash/vk/struct.AccessFlags.html>"]
///
#[doc = "Vulkan documentation: <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkAccessFlagBits.html>"]
pub type AccessFlags = vk::AccessFlags;

pub const SUBPASS_EXTERNAL: u32 = vk::SUBPASS_EXTERNAL;

pub const NO_ATTACHMENT: u32 =  vk::ATTACHMENT_UNUSED;

/// Essentially SubpassSync acts like a memory barrier between two (previous and next) subpasses
#[derive(Debug)]
pub struct SubpassSync {
    /// Index of previous subpass in [`RenderPassType::subpasses`] or [`SUBPASS_EXTERNAL`]
    pub src_subpass: u32,
    /// Index of next subpass in [`RenderPassType::subpasses`] or [`SUBPASS_EXTERNAL`]
    pub dst_subpass: u32,
    /// Pipeline stage during which a given attachment was used before
    pub src_stage: PipelineStage,
    /// Pipeline stage during which a given attachment will be used later
    pub dst_stage: PipelineStage,
    /// Types of memory operations that occurred in a src subpass or before a render pass
    pub src_access: AccessFlags,
    /// Types of memory operations that occurred in a dst subpass or after a render pass
    pub dst_access: AccessFlags,
}

#[doc(hidden)]
impl From<&SubpassSync> for vk::SubpassDependency {
    fn from(sync: &SubpassSync) -> vk::SubpassDependency {
        vk::SubpassDependency {
            src_subpass: sync.src_subpass,
            dst_subpass: sync.dst_subpass,
            src_stage_mask: sync.src_stage,
            dst_stage_mask: sync.dst_stage,
            src_access_mask: sync.src_access,
            dst_access_mask: sync.dst_access,
            dependency_flags: vk::DependencyFlags::BY_REGION,
        }
    }
}

#[derive(Debug)]
struct SubpassView {
    pub depth_attachment: vk::AttachmentReference,
    pub resolve_attachment: Vec<vk::AttachmentReference>,
    pub color_attachment: Vec<vk::AttachmentReference>,
    pub input_attachment: Vec<vk::AttachmentReference>,
    pub preserve_attachments: Vec<u32>,
}

#[doc(hidden)]
impl From<&SubpassView> for vk::SubpassDescription {
    fn from(view: &SubpassView) -> Self {
        vk::SubpassDescription {
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: view.input_attachment.len() as u32,
            p_input_attachments: data_ptr!(view.input_attachment),
            color_attachment_count: view.color_attachment.len() as u32,
            p_color_attachments: data_ptr!(view.color_attachment),
            p_resolve_attachments: data_ptr!(view.resolve_attachment),
            p_depth_stencil_attachment: &view.depth_attachment,
            preserve_attachment_count: view.preserve_attachments.len() as u32,
            p_preserve_attachments: data_ptr!(view.preserve_attachments),
        }
    }
}

/// `Subpass` configuration
///
/// All information about [valid usage](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkSubpassDescription.html)
///
/// Note: [`SubpassInfo::resolve_attachments`] **must be** `&[]` or same length as [`SubpassInfo::color_attachments`]
#[derive(Debug)]
pub struct SubpassInfo<'a> {
    pub input_attachments: &'a [u32],
    pub color_attachments: &'a [u32],
    pub resolve_attachments: &'a [u32],
    pub depth_stencil_attachment: u32,
    pub preserve_attachments: &'a [u32],
}

impl<'a> Default for SubpassInfo<'a> {
    fn default() -> SubpassInfo<'a> {
        SubpassInfo {
            input_attachments: &[],
            color_attachments: &[],
            resolve_attachments: &[],
            depth_stencil_attachment: NO_ATTACHMENT,
            preserve_attachments: &[],
        }
    }
}

#[doc(hidden)]
impl From<&SubpassInfo<'_>> for SubpassView {
    fn from(info: &SubpassInfo) -> Self {
        let input_attch: Vec<vk::AttachmentReference> = info
            .input_attachments
            .iter()
            .map(|&i| vk::AttachmentReference {
                attachment: i,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            })
            .collect();

        let color_attch: Vec<vk::AttachmentReference> = info
            .color_attachments
            .iter()
            .map(|&i| vk::AttachmentReference {
                attachment: i,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            })
            .collect();

        let resolve_attch: Vec<vk::AttachmentReference> = info
            .resolve_attachments
            .iter()
            .map(|&i| vk::AttachmentReference {
                attachment: i,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            })
            .collect();

        let depth_attch = vk::AttachmentReference {
            attachment: info.depth_stencil_attachment,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        SubpassView {
            depth_attachment: depth_attch,
            resolve_attachment: resolve_attch,
            color_attachment: color_attch,
            input_attachment: input_attch,
            preserve_attachments: info.preserve_attachments.to_vec(),
        }
    }
}

#[derive(Debug)]
pub enum RenderPassError {
    /// Error was returned as a result of `vkCreateRenderPass`
    /// [call](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateRenderPass.html)
    Creation,
}

impl fmt::Display for RenderPassError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vkCreateRenderPass call failed")
    }
}

impl Error for RenderPassError { }

/// [`RenderPass`] configuration
pub struct RenderPassType<'a> {
    pub device: &'a dev::Device<'a>,
    pub attachments: &'a [AttachmentInfo],
    pub sync_info: &'a [SubpassSync],
    pub subpasses: &'a [SubpassInfo<'a>],
}

/// Context for executing graphics pipeline
pub struct RenderPass<'a> {
    i_dev: &'a dev::Device<'a>,
    i_rp: vk::RenderPass,
}

impl<'a> RenderPass<'a> {
    pub fn new(rp_type: &'a RenderPassType) -> Result<RenderPass<'a>, RenderPassError> {
        let dependencies: Vec<vk::SubpassDependency> = rp_type
            .sync_info
            .iter()
            .map(|x| x.into())
            .collect();

        let attachments: Vec<vk::AttachmentDescription> = rp_type
            .attachments
            .iter()
            .map(|x| x.into())
            .collect();

        let subpasses_slice: Vec<SubpassView> = rp_type
            .subpasses
            .iter()
            .map(|x| x.into())
            .collect();

        let subpasses: Vec<vk::SubpassDescription> = subpasses_slice
            .iter()
            .map(|x| x.into())
            .collect();

        let render_pass_create_info:vk::RenderPassCreateInfo = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::RenderPassCreateFlags::empty(),
            attachment_count: attachments.len() as u32,
            p_attachments: data_ptr!(attachments),
            subpass_count: subpasses.len() as u32,
            p_subpasses: data_ptr!(subpasses),
            dependency_count: dependencies.len() as u32,
            p_dependencies: data_ptr!(dependencies),
        };

        let rp = on_error_ret!(
            unsafe { rp_type.device.device().create_render_pass(&render_pass_create_info, None) },
            RenderPassError::Creation
        );

        Ok(
            RenderPass {
                i_dev: rp_type.device,
                i_rp: rp,
            }
        )
    }
}

impl<'a> Drop for RenderPass<'a> {
    fn drop(&mut self) {
        unsafe {
            self.i_dev.device().destroy_render_pass(self.i_rp, None);
        }
    }
}
