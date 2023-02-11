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
use std::sync::Arc;
use std::error::Error;
use std::convert::Into;

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

/// [`RenderPass`] configuration
pub struct RenderPassCfg<'a, 'b: 'a> {
    pub attachments: &'a [AttachmentInfo],
    pub sync_info: &'a [SubpassSync],
    pub subpasses: &'a [SubpassInfo<'b>],
}

/// Context for executing graphics pipeline
pub struct RenderPass {
    i_core: Arc<dev::Core>,
    i_rp: vk::RenderPass
}

impl RenderPass {
    pub fn new(dev: &dev::Device, cfg: &RenderPassCfg) -> Result<RenderPass, RenderPassError> {
        let dependencies: Vec<vk::SubpassDependency> = cfg
            .sync_info
            .iter()
            .map(|x| x.into())
            .collect();

        let attachments: Vec<vk::AttachmentDescription> = cfg
            .attachments
            .iter()
            .map(|x| x.into())
            .collect();

        let input_attch: Vec<Vec<vk::AttachmentReference>> = cfg
            .subpasses
            .iter()
            .map(|x| {
                x.input_attachments.iter().map(|&i| vk::AttachmentReference {
                    attachment: i,
                    layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                })
                .collect()
            })
            .collect();

        let color_attch: Vec<Vec<vk::AttachmentReference>> = cfg
            .subpasses
            .iter()
            .map(|x| {
                x.color_attachments.iter().map(|&i| vk::AttachmentReference {
                    attachment: i,
                    layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                })
                .collect()
            })
            .collect();

        let resolve_attch: Vec<Vec<vk::AttachmentReference>> = cfg
            .subpasses
            .iter()
            .map(|x| {
                x.resolve_attachments.iter().map(|&i| vk::AttachmentReference {
                    attachment: i,
                    layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                })
                .collect()
            })
            .collect();

        let depth_attch: Vec<vk::AttachmentReference> = cfg
            .subpasses
            .iter()
            .map(|x| vk::AttachmentReference {
                attachment: x.depth_stencil_attachment,
                layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            })
            .collect();

            let subpasses: Vec<vk::SubpassDescription> = cfg
                .subpasses
                .iter()
                .enumerate()
                .map(|(i, x)| vk::SubpassDescription {
                    flags: vk::SubpassDescriptionFlags::empty(),
                    pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
                    input_attachment_count: input_attch[i].len() as u32,
                    p_input_attachments: data_ptr!(input_attch[i]),
                    color_attachment_count: color_attch[i].len() as u32,
                    p_color_attachments: data_ptr!(color_attch[i]),
                    p_resolve_attachments: data_ptr!(resolve_attch[i]),
                    p_depth_stencil_attachment: &depth_attch[i],
                    preserve_attachment_count: x.preserve_attachments.len() as u32,
                    p_preserve_attachments: data_ptr!(x.preserve_attachments),
                })
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
            unsafe { dev.device().create_render_pass(&render_pass_create_info, dev.core().allocator()) },
            RenderPassError::Creation
        );

        Ok(
            RenderPass {
                i_core: dev.core().clone(),
                i_rp: rp
            }
        )
    }

    /// Create [`RenderPass`] with single subpass and single attachment
    pub fn single_subpass(device: &dev::Device, img_format: surface::ImageFormat)
        -> Result<RenderPass, RenderPassError>
    {
        let subpass_info = [
            SubpassInfo {
                input_attachments: &[],
                color_attachments: &[0],
                resolve_attachments: &[],
                depth_stencil_attachment: NO_ATTACHMENT,
                preserve_attachments: &[],
            }
        ];

        let attachments = [
            AttachmentInfo {
                format: img_format,
                load_op: AttachmentLoadOp::CLEAR,
                store_op: AttachmentStoreOp::STORE,
                stencil_load_op: AttachmentLoadOp::DONT_CARE,
                stencil_store_op: AttachmentStoreOp::DONT_CARE,
                initial_layout: ImageLayout::UNDEFINED,
                final_layout: ImageLayout::PRESENT_SRC_KHR,
            }
        ];

        let subpass_sync_info = [
            SubpassSync {
                src_subpass: SUBPASS_EXTERNAL,
                dst_subpass: 0,
                src_stage: PipelineStage::BOTTOM_OF_PIPE,
                dst_stage: PipelineStage::COLOR_ATTACHMENT_OUTPUT,
                src_access: AccessFlags::MEMORY_READ,
                dst_access: AccessFlags::COLOR_ATTACHMENT_WRITE | AccessFlags::COLOR_ATTACHMENT_READ,
            },
            SubpassSync {
                src_subpass: 0,
                dst_subpass: SUBPASS_EXTERNAL,
                src_stage: PipelineStage::COLOR_ATTACHMENT_OUTPUT,
                dst_stage: PipelineStage::BOTTOM_OF_PIPE,
                src_access: AccessFlags::COLOR_ATTACHMENT_WRITE | AccessFlags::COLOR_ATTACHMENT_READ,
                dst_access: AccessFlags::MEMORY_READ,
            }
        ];

        let rp_cfg = RenderPassCfg {
            attachments: &attachments,
            sync_info: &subpass_sync_info,
            subpasses: &subpass_info,
        };

        RenderPass::new(&device, &rp_cfg)
    }

    #[doc(hidden)]
    pub fn render_pass(&self) -> vk::RenderPass {
        self.i_rp
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            self.i_core.device().destroy_render_pass(self.i_rp, self.i_core.allocator());
        }
    }
}