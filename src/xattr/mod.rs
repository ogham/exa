//! Extended attribute support
#[cfg(target_os = "macos")]
mod xattr_darwin;
#[cfg(target_os = "macos")]
pub use self::xattr_darwin::*;
#[cfg(target_os = "linux")]
mod xattr_linux;
#[cfg(target_os = "linux")]
pub use self::xattr_linux::*;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
mod xattr_other;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub use self::xattr_other::*;