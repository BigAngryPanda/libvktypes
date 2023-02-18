//! Provide shader handler type

use ash::vk;
use ash::util::read_spv;

use crate::dev;
use crate::on_error_ret;

use std::{ptr, mem, fmt};
use std::error::Error;
use std::sync::Arc;
use std::fs::File;
use std::path::Path;
use std::ffi::CString;

use shaderc;

pub type ShaderType = shaderc::ShaderKind;

pub struct ShaderCfg<'a> {
    pub path: &'a str,
    pub entry: &'a str,
}

#[derive(Debug)]
pub enum ShaderError {
	InvalidFile,
	BytecodeRead,
	ShaderCreation,
    NullTerminate
}

impl fmt::Display for ShaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = match self {
            ShaderError::InvalidFile => {
                "Failed to open file"
            },
            ShaderError::BytecodeRead => {
                "Failed to read from file"
            },
            ShaderError::ShaderCreation => {
                "Failed to create shader (vkCreateShaderModule call failed)"
            },
            ShaderError::NullTerminate => {
                "Failed to null terminate shader entry name"
            }
        };

        write!(f, "{:?}", err_msg)
    }
}

impl Error for ShaderError {}

/// Shader type represents loaded shader bytecode wrapper
///
/// You may think of it as file handler
pub struct Shader {
	i_core: Arc<dev::Core>,
	i_module: vk::ShaderModule,
	i_entry: CString,
}

impl Shader {
    pub fn from_bytecode(device: &dev::Device, shader_type: &ShaderCfg, bytecode: &[u32]) -> Result<Shader, ShaderError> {
        let shader_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: bytecode.len()*mem::size_of::<u32>(),
            p_code: bytecode.as_ptr(),
        };

        let shader_module: vk::ShaderModule = on_error_ret!(
            unsafe { device.device().create_shader_module(&shader_info, device.allocator()) },
            ShaderError::ShaderCreation
        );

        let entry = on_error_ret!(CString::new(shader_type.entry), ShaderError::NullTerminate);

        Ok(Shader {
            i_core: device.core().clone(),
            i_module: shader_module,
            i_entry: entry
        })
    }

    pub fn from_file(device: &dev::Device, shader_type: &ShaderCfg) -> Result<Shader, ShaderError> {
        let mut spv_file: File = on_error_ret!(
            File::open(Path::new(shader_type.path)),
            ShaderError::InvalidFile
        );

        let spv_bytecode: Vec<u32> = on_error_ret!(
            read_spv(&mut spv_file),
            ShaderError::BytecodeRead
        );

        Shader::from_bytecode(device, shader_type, &spv_bytecode)
    }

    /// Return reference to name of entry function (point) in shader
    pub fn entry(&self) -> &CString {
        &self.i_entry
    }

    #[doc(hidden)]
    pub fn module(&self) -> vk::ShaderModule {
        self.i_module
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.i_core.device().destroy_shader_module(self.i_module, self.i_core.allocator());
        }
    }
}