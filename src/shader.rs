//! Provide shader handler type

use ash::vk;
use ash::util::read_spv;

use crate::dev;
use crate::on_error_ret;

use std::{ptr, mem, fmt};
use std::error::Error;
use std::sync::Arc;
use std::fs;
use std::path::Path;
use std::ffi::CString;
use std::marker::PhantomData;

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

pub struct ShaderBuilder<'a, 'b, 'c, 'd> {
    path: &'a str,
    entry:  &'b str,
    bytecode: &'c [u32],
    glsl_src: &'d str,
    kind: Kind
}

impl<'a, 'b, 'c, 'd> ShaderBuilder<'a, 'b, 'c, 'd> {
    const DEFAULT_ENTRY: &'static str = "main";
    const DEFAULT_PATH: &'static str = "default_path";
    const SRC: &'static str = "";
    const BYTECODE: &'static [u32] = &[];

    pub fn new() -> ShaderBuilder<'a, 'b, 'c, 'd> {
        ShaderBuilder {
            path: Self::DEFAULT_PATH,
            entry: Self::DEFAULT_ENTRY,
            bytecode: Self::BYTECODE,
            glsl_src: Self::SRC,
            kind: Kind::Vertex
        }
    }

    /// Must be called if you create shader from SPIR-V [file](Self::from_file)
    ///
    /// Default is `"default_path"`
    pub fn path(mut self, new_path: &'a str) -> Self {
        self.path = new_path;

        self
    }

    /// Must be called if you create shader from [bytecode](Self::from_bytecode)
    ///
    /// Default is empty
    pub fn bytecode(mut self, new_bytecode: &'c [u32]) -> Self {
        self.bytecode = new_bytecode;

        self
    }

    /// Must be called if you crate shader from glsl source [code](Self::from_glsl) or [file](Self::from_glsl_file)
    ///
    /// It is `Kind::Vertex` by default
    pub fn shader_type(mut self, shader_type: Kind) -> Self {
        self.kind = shader_type;

        self
    }

    /// Must be called if you crate shader from glsl source [code](Self::from_glsl)
    ///
    /// It is empty by default
    pub fn glsl_src(mut self, src: &'d str) -> Self {
        self.glsl_src = src;

        self
    }

    /// Optional
    ///
    /// Default is `"main"`
    pub fn entry(mut self, new_entry: &'b str) -> Self {
        self.entry = new_entry;

        self
    }

    /// Build shader module from provided SPIR-V bytecode
    pub fn from_bytecode(self, device: &dev::Device) -> Result<Shader, ShaderError> {
        let shader_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: self.bytecode.len()*mem::size_of::<u32>(),
            p_code: self.bytecode.as_ptr(),
            _marker: PhantomData,
        };

        let shader_module: vk::ShaderModule = on_error_ret!(
            unsafe { device.device().create_shader_module(&shader_info, device.allocator()) },
            ShaderError::ShaderCreation
        );

        let entry = on_error_ret!(CString::new(self.entry), ShaderError::NullTerminate);

        Ok(Shader {
            i_core: device.core().clone(),
            i_module: shader_module,
            i_entry: entry
        })
    }

    /// Build shader module from SPIR-V bytecode file
    ///
    /// Note: compare this method with [`from_glsl_file`](Self::from_glsl_file)
    pub fn from_file(self, device: &dev::Device) -> Result<Shader, ShaderError> {
        let mut spv_file: fs::File = on_error_ret!(
            fs::File::open(Path::new(self.path)),
            ShaderError::InvalidFile
        );

        let spv_bytecode: Vec<u32> = on_error_ret!(
            read_spv(&mut spv_file),
            ShaderError::BytecodeRead
        );

        self.bytecode(&spv_bytecode).from_bytecode(device)
    }

    /// Build shader module from `glsl` source code directly
    pub fn from_glsl(self, device: &dev::Device) -> Result<Shader, ShaderError> {
        let compiler = on_error_ret!(shaderc::Compiler::new(), ShaderError::Shaderc);

        let binary_result = match compiler.compile_into_spirv(self.glsl_src, self.kind, self.path, self.entry, None) {
            Ok(val) => val,
            Err(err) => {
                print!("{}", err);
                return Err(ShaderError::Compiling);
            }
        };

        if binary_result.is_empty() {
            return Err(ShaderError::Compiling);
        }

        self.bytecode(binary_result.as_binary()).from_bytecode(device)
    }

    /// Build shader module from file with `glsl` source code directly
    ///
    /// Note: compare this method with [`from_file`](Self::from_file)
    pub fn from_glsl_file(self, device: &dev::Device) -> Result<Shader, ShaderError> {
        let src = on_error_ret!(fs::read_to_string(self.path), ShaderError::InvalidFile);

        self.glsl_src(&src).from_glsl(device)
    }
}

/// Shader type represents loaded shader bytecode wrapper
///
/// You may think of it as file handler
pub struct Shader {
    i_core: Arc<dev::Core>,
    i_module: vk::ShaderModule,
    i_entry: CString,
}

impl Shader {
    /// Return reference to name of entry function (point) in shader
    pub fn entry(&self) -> &CString {
        &self.i_entry
    }

    pub(crate) fn module(&self) -> vk::ShaderModule {
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
