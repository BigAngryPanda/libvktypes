//! Key module in the library
//!
//! Contains structs which allow you work with GPU

pub mod device;

#[doc(hidden)]
pub mod core;

pub use device::*;

#[doc(hidden)]
pub(crate) use self::core::*;