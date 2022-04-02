//! Provide specialization constants API

use ash::vk;

use std::ffi::c_void;
use std::mem::size_of;
use std::ptr;

pub struct SpecializationConstant {
	i_map:  Vec<vk::SpecializationMapEntry>,
	i_info: vk::SpecializationInfo,
}

impl<'a> SpecializationConstant {
	pub fn new<T: Sized>(data: &'a [T]) -> SpecializationConstant {
		let mut map_entry: Vec<vk::SpecializationMapEntry> = Vec::with_capacity(data.len());

		for i in 0..data.len() {
			map_entry.push(
				vk::SpecializationMapEntry {
					constant_id: i as u32,
					offset: (i*size_of::<T>()) as u32,
					size: size_of::<T>(),
				}
			);
		}

		let info = if data.is_empty() {
			vk::SpecializationInfo {
				map_entry_count: map_entry.len() as u32,
				p_map_entries: map_entry.as_ptr(),
				data_size: size_of::<T>()*data.len(),
				p_data: data.as_ptr() as *const c_void
			}
		}
		else {
			vk::SpecializationInfo {
				map_entry_count: 0,
				p_map_entries: ptr::null(),
				data_size: 0,
				p_data: ptr::null()
			}
		};

		SpecializationConstant {
			i_map: map_entry,
			i_info: info
		}
	}

	#[doc(hidden)]
	pub fn info(&self) -> &vk::SpecializationInfo {
		&self.i_info
	}
}