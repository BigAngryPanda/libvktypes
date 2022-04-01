//! Provide specialization constants API

use ash::vk;

use std::ffi::c_void;
use std::mem::size_of;

pub struct SpecializationConstant {
	i_data_ptr: *const c_void,
	i_map:  vk::SpecializationMapEntry
}

impl<'a> SpecializationConstant {
	pub fn new<T: Sized>(data: &'a T) -> SpecializationConstant {
		// Sort of layout for every constant
		SpecializationConstant {
			i_data_ptr: data as *const T as *const c_void,
			i_map: vk::SpecializationMapEntry {
				constant_id: 0,
				offset: 0,
				size: size_of::<T>(),
			}
		}
	}

	#[doc(hidden)]
	pub fn info(&self) -> vk::SpecializationInfo {
		vk::SpecializationInfo {
			map_entry_count: 1,
			p_map_entries: &self.i_map,
			data_size: self.i_map.size,
			p_data: self.i_data_ptr
		}
	}
}