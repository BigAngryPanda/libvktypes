use crate::types::layers::DebugLayer;

use std::ffi::CStr;

#[derive(Debug)]
pub struct LibInstanceType<'a> {
    pub version_major: u32,
    pub version_minor: u32,
    pub version_patch: u32,
    pub dynamic_load: bool,
    pub debug_layer: Option<DebugLayer>,
    pub extensions: &'a [&'a CStr],
}

impl<'a> Default for LibInstanceType<'a> {
    fn default() -> LibInstanceType<'a> {
        LibInstanceType {
            version_major: 1,
            version_minor: 0,
            version_patch: 0,
            dynamic_load: false,
            debug_layer: None,
            extensions: &[],
        }
    }
}