//! Provide shader handler type

use ash::vk;
use ash::util::read_spv;

use crate::dev;
use crate::{on_error_ret, on_option_ret};

use std::{ptr, mem, fmt};
use std::error::Error;
use std::sync::Arc;
use std::fs;
use std::path::Path;
use std::ffi::CString;

use shaderc;

/// See
/// [documentation](https://docs.rs/shaderc/latest/shaderc/enum.ShaderKind.html)
/// about possible values
/// Example
/// ```
/// use libvktypes::shader;
///
/// // Example for vertex shader
/// let vertex_shader_type = shader::Kind::Vertex;
///
/// // And so on
/// let fragment_shader_type = shader::Kind::Fragment;
///
/// let compute_shader_type = shader::Kind::Compute;
///
/// let geometry_shader_type = shader::Kind::Geometry;
/// ```
pub type Kind = shaderc::ShaderKind;

pub struct ShaderCfg<'a> {
    pub path: &'a str,
    pub entry: &'a str,
}

#[derive(Debug)]
pub enum ShaderError {
	InvalidFile,
	BytecodeRead,
	ShaderCreation,
    Shaderc,
    Compiling,
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
            ShaderError::Shaderc => {
                "Failed to create compiler (internal shaderc library error)"
            },
            ShaderError::Compiling => {
                "Failed to compile shader source code"
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
    /// Build shader module from provided SPIR-V bytecode
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

    /// Build shader module from SPIR-V bytecode file
    ///
    /// Note: compare this method with [`from_glsl_file`](Self::from_glsl_file)
    pub fn from_file(device: &dev::Device, shader_type: &ShaderCfg) -> Result<Shader, ShaderError> {
        let mut spv_file: fs::File = on_error_ret!(
            fs::File::open(Path::new(shader_type.path)),
            ShaderError::InvalidFile
        );

        let spv_bytecode: Vec<u32> = on_error_ret!(
            read_spv(&mut spv_file),
            ShaderError::BytecodeRead
        );

        Shader::from_bytecode(device, shader_type, &spv_bytecode)
    }

    /// Build shader module from `glsl` source code directly
    pub fn from_glsl(device: &dev::Device, cfg: &ShaderCfg, src: &str, kind: Kind) -> Result<Shader, ShaderError> {
        let compiler = on_option_ret!(shaderc::Compiler::new(), ShaderError::Shaderc);

        let binary_result = on_error_ret!(
            compiler.compile_into_spirv(src, kind, cfg.path, cfg.entry, None),
            ShaderError::Compiling
        );

        if binary_result.is_empty() {
            return Err(ShaderError::Compiling);
        }

        Self::from_bytecode(device, cfg, binary_result.as_binary())
    }

    /// Build shader module from file with `glsl` source code directly
    ///
    /// Note: compare this method with [`from_file`](Self::from_file)
    pub fn from_glsl_file(device: &dev::Device, cfg: &ShaderCfg, kind: Kind) -> Result<Shader, ShaderError> {
        let src = on_error_ret!(fs::read_to_string(cfg.path), ShaderError::InvalidFile);

        Self::from_glsl(device, cfg, &src, kind)
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