//! Provide specialization constants API

use ash::vk;

pub struct SpecializationConstant<T: Sized> {
	i_data: T,
	i_map:  vk::SpecializationMapEntry,
}

impl<'a, T> SpecializationConstant<T> {
	pub fn new(data: T) -> SpecializationConstant<T> {
		// Sort of layout for every constant
		SpecializationConstant {
			i_data: data,
			i_map: vk::SpecializationMapEntry {
				constant_id: 0,
				offset: 0,
				size: std::mem::size_of::<T>(),
			}
		}
	}

	pub fn empty() -> SpecializationConstant<()> {
		SpecializationConstant {
			i_data: (),
			i_map: vk::SpecializationMapEntry {
				constant_id: 0,
				offset: 0,
				size: 0,
			}
		}
	}

	#[doc(hidden)]
	pub fn info(&self) -> vk::SpecializationInfo {
		vk::SpecializationInfo {
			map_entry_count: 1,
			p_map_entries: &self.i_map,
			data_size: self.i_map.size,
			p_data: &self.i_data as *const T as *const std::ffi::c_void
		}
	}
}