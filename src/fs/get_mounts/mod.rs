#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use self::linux::get_mount_points;

#[cfg(all(unix, not(target_os = "linux")))]
mod bsd;
#[cfg(all(unix, not(target_os = "linux")))]
pub use self::bsd::get_mount_points;
