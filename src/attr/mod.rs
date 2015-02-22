//! Extended attribute support
#[cfg(target_os = "macos")]
mod attr_darwin;
#[cfg(target_os = "macos")]
pub use self::attr_darwin::*;
#[cfg(not(target_os = "macos"))]
mod attr_other;
#[cfg(not(target_os = "macos"))]
pub use self::attr_other::*;

