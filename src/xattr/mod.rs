//! Extended attribute support
#[cfg(target_os = "macos")]
mod xattr_darwin;
#[cfg(target_os = "macos")]
pub use self::xattr_darwin::*;
#[cfg(not(target_os = "macos"))]
mod xattr_other;
#[cfg(not(target_os = "macos"))]
pub use self::xattr_other::*;

