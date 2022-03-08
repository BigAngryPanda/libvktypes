//! Provide shader handler type

use ash::vk;

use ash::util::read_spv;

use crate::logical_device::LogicalDevice;
use crate::on_error;

use std::{
	ptr,
	mem
};

use std::fs::File;
use std::path::Path;

/// Shader type represents loaded shader bytecode wrapper
/// You may think of it as file handler
#[derive(Debug)]
pub struct Shader {
	pub i_module: vk::ShaderModule,
	pub i_entry: String
}

#[derive(Debug)]
pub enum ShaderError {
	InvalidFile,
	BytecodeRead,
	ShaderCreation,
}

impl Shader {
	pub fn from_bytecode(dev: &LogicalDevice, bytecode: &[u32], entry: String) -> Result<Shader, ShaderError> {
		let shader_info = vk::ShaderModuleCreateInfo {
			s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
			p_next: ptr::null(),
			flags: vk::ShaderModuleCreateFlags::empty(),
			code_size: bytecode.len()*mem::size_of::<u32>(),
			p_code: bytecode.as_ptr(),
		};

		let shader_module:vk::ShaderModule = on_error!(
			unsafe { dev.i_device.create_shader_module(&shader_info, None) },
			return Err(ShaderError::ShaderCreation)
		);

		let result = Shader {
			i_module: shader_module,
			i_entry:  entry
		};

		Ok(result)
	}

	pub fn from_src(dev: &LogicalDevice, path: &str, entry: String) -> Result<Shader, ShaderError> {
		let mut spv_file:File = on_error!(
			File::open(Path::new(path)),
			return Err(ShaderError::InvalidFile)
		);

		let spv_bytecode:Vec<u32> = on_error!(
			read_spv(&mut spv_file),
			return Err(ShaderError::BytecodeRead)
		);

		Shader::from_bytecode(dev, &spv_bytecode, entry)
	}
}