//! Functional objects definitions such as Instance and Device
//!
//! Instead of types `resources` are more sophisticated and carry some internal logic (instead of types which is pure description)
//!
//! Here `resource` and `object` is used interchangeably
//!
//! Usually each object created via providing corresponded object type

pub mod macros;
pub mod libvk;
pub mod hw;
pub mod dev;
pub mod layers;
pub mod extensions;
pub mod debug;
pub mod memory;
pub mod shader;
pub mod compute;
pub mod cmd;
pub mod surface;
pub mod window;