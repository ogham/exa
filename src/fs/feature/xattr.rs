//! Extended attribute support for Darwin and Linux systems.

#![allow(trivial_casts)]  // for ARM

use std::cmp::Ordering;
use std::ffi::CString;
use std::io;
use std::path::Path;


pub const ENABLED: bool = cfg!(any(target_os = "macos", target_os = "linux"));


pub trait FileAttributes {
    fn attributes(&self) -> io::Result<Vec<Attribute>>;
    fn symlink_attributes(&self) -> io::Result<Vec<Attribute>>;
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl FileAttributes for Path {
    fn attributes(&self) -> io::Result<Vec<Attribute>> {
        list_attrs(&lister::Lister::new(FollowSymlinks::Yes), self)
    }

    fn symlink_attributes(&self) -> io::Result<Vec<Attribute>> {
        list_attrs(&lister::Lister::new(FollowSymlinks::No), self)
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
impl FileAttributes for Path {
    fn attributes(&self) -> io::Result<Vec<Attribute>> {
        Ok(Vec::new())
    }

    fn symlink_attributes(&self) -> io::Result<Vec<Attribute>> {
        Ok(Vec::new())
    }
}


/// Attributes which can be passed to `Attribute::list_with_flags`
#[cfg(any(target_os = "macos", target_os = "linux"))]
#[derive(Copy, Clone)]
pub enum FollowSymlinks {
    Yes,
    No,
}

/// Extended attribute
#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}


#[cfg(any(target_os = "macos", target_os = "linux"))]
fn get_secattr(lister: &lister::Lister, c_path: &std::ffi::CString) -> io::Result<Vec<Attribute>> {
    const SELINUX_XATTR_NAME: &str = "security.selinux";
    const ENODATA: i32 = 61;

    let c_attr_name = CString::new(SELINUX_XATTR_NAME).map_err(|e| {
        io::Error::new(io::ErrorKind::Other, e)
    })?;
    let size = lister.getxattr_first(c_path, &c_attr_name);

    let size = match size.cmp(&0) {
        Ordering::Less => {
            let e = io::Error::last_os_error();

            if e.kind() == io::ErrorKind::Other && e.raw_os_error() == Some(ENODATA) {
                return Ok(Vec::new())
            }

            return Err(e)
        },
        Ordering::Equal => return Err(io::Error::from(io::ErrorKind::InvalidData)),
        Ordering::Greater => size as usize,
    };

    let mut buf_value = vec![0_u8; size];
    let size = lister.getxattr_second(c_path, &c_attr_name, &mut buf_value, size);

    match size.cmp(&0) {
        Ordering::Less => return Err(io::Error::last_os_error()),
        Ordering::Equal => return Err(io::Error::from(io::ErrorKind::InvalidData)),
        Ordering::Greater => (),
    }

    Ok(vec![Attribute {
        name:  String::from(SELINUX_XATTR_NAME),
        value: lister.translate_attribute_data(&buf_value),
    }])
}

pub fn list_attrs(lister: &lister::Lister, path: &Path) -> io::Result<Vec<Attribute>> {
    let c_path = CString::new(path.to_str().ok_or(io::Error::new(io::ErrorKind::Other, "Error: path not convertible to string"))?).map_err(|e| {
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    let bufsize = lister.listxattr_first(&c_path);
    let bufsize = match bufsize.cmp(&0) {
        Ordering::Less     => return Err(io::Error::last_os_error()),
        // Some filesystems, like sysfs, return nothing on listxattr, even though the security
        // attribute is set.
        Ordering::Equal    => return get_secattr(lister, &c_path),
        Ordering::Greater  => bufsize as usize,
    };

    let mut buf = vec![0_u8; bufsize];

    match lister.listxattr_second(&c_path, &mut buf, bufsize).cmp(&0) {
        Ordering::Less     => return Err(io::Error::last_os_error()),
        Ordering::Equal    => return Ok(Vec::new()),
        Ordering::Greater  => {},
    }

    let mut names = Vec::new();

    for attr_name in buf.split(|c| c == &0) {
        if attr_name.is_empty() {
            continue;
        }

        let c_attr_name = CString::new(attr_name).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, e)
        })?;
        let size = lister.getxattr_first(&c_path, &c_attr_name);

        if size > 0 {
            let mut buf_value = vec![0_u8; size as usize];
            if lister.getxattr_second(&c_path, &c_attr_name, &mut buf_value, size as usize) < 0 {
                return Err(io::Error::last_os_error());
            }

            names.push(Attribute {
                name:  lister.translate_attribute_data(attr_name),
                value: lister.translate_attribute_data(&buf_value),
            });
        } else {
            names.push(Attribute {
                name:  lister.translate_attribute_data(attr_name),
                value: String::new(),
            });
        }
    }

    Ok(names)
}


#[cfg(target_os = "macos")]
mod lister {
    use super::FollowSymlinks;
    use libc::{c_int, size_t, ssize_t, c_char, c_void};
    use std::ffi::CString;
    use std::ptr;

    extern "C" {
        fn listxattr(
            path: *const c_char,
            namebuf: *mut c_char,
            size: size_t,
            options: c_int,
        ) -> ssize_t;

        fn getxattr(
            path: *const c_char,
            name: *const c_char,
            value: *mut c_void,
            size: size_t,
            position: u32,
            options: c_int,
        ) -> ssize_t;
    }

    pub struct Lister {
        c_flags: c_int,
    }

    impl Lister {
        pub fn new(do_follow: FollowSymlinks) -> Self {
            let c_flags: c_int = match do_follow {
                FollowSymlinks::Yes  => 0x0001,
                FollowSymlinks::No   => 0x0000,
            };

            Self { c_flags }
        }

        pub fn translate_attribute_data(&self, input: &[u8]) -> String {
            unsafe { std::str::from_utf8_unchecked(input).trim_end_matches('\0').into() }
        }

        pub fn listxattr_first(&self, c_path: &CString) -> ssize_t {
            unsafe {
                listxattr(
                    c_path.as_ptr(),
                    ptr::null_mut(),
                    0,
                    self.c_flags,
                )
            }
        }

        pub fn listxattr_second(&self, c_path: &CString, buf: &mut [u8], bufsize: size_t) -> ssize_t {
            unsafe {
                listxattr(
                    c_path.as_ptr(),
                    buf.as_mut_ptr().cast(),
                    bufsize,
                    self.c_flags,
                )
            }
        }

        pub fn getxattr_first(&self, c_path: &CString, c_name: &CString) -> ssize_t {
            unsafe {
                getxattr(
                    c_path.as_ptr(),
                    c_name.as_ptr().cast(),
                    ptr::null_mut(),
                    0,
                    0,
                    self.c_flags,
                )
            }
        }

        pub fn getxattr_second(&self, c_path: &CString, c_name: &CString, buf: &mut [u8], bufsize: size_t) -> ssize_t {
            unsafe {
                getxattr(
                    c_path.as_ptr(),
                    c_name.as_ptr().cast(),
                    buf.as_mut_ptr().cast::<libc::c_void>(),
                    bufsize,
                    0,
                    self.c_flags,
                )
            }
        }
    }
}


#[cfg(target_os = "linux")]
mod lister {
    use std::ffi::CString;
    use libc::{size_t, ssize_t, c_char, c_void};
    use super::FollowSymlinks;
    use std::ptr;

    extern "C" {
        fn listxattr(
            path: *const c_char,
            list: *mut c_char,
            size: size_t,
        ) -> ssize_t;

        fn llistxattr(
            path: *const c_char,
            list: *mut c_char,
            size: size_t,
        ) -> ssize_t;

        fn getxattr(
            path: *const c_char,
            name: *const c_char,
            value: *mut c_void,
            size: size_t,
        ) -> ssize_t;

        fn lgetxattr(
            path: *const c_char,
            name: *const c_char,
            value: *mut c_void,
            size: size_t,
        ) -> ssize_t;
    }

    pub struct Lister {
        follow_symlinks: FollowSymlinks,
    }

    impl Lister {
        pub fn new(follow_symlinks: FollowSymlinks) -> Lister {
            Lister { follow_symlinks }
        }

        pub fn translate_attribute_data(&self, input: &[u8]) -> String {
            String::from_utf8_lossy(input).trim_end_matches('\0').into()
        }

        pub fn listxattr_first(&self, c_path: &CString) -> ssize_t {
            let listxattr = match self.follow_symlinks {
                FollowSymlinks::Yes  => listxattr,
                FollowSymlinks::No   => llistxattr,
            };

            unsafe {
                listxattr(
                    c_path.as_ptr(),
                    ptr::null_mut(),
                    0,
                )
            }
        }

        pub fn listxattr_second(&self, c_path: &CString, buf: &mut [u8], bufsize: size_t) -> ssize_t {
            let listxattr = match self.follow_symlinks {
                FollowSymlinks::Yes  => listxattr,
                FollowSymlinks::No   => llistxattr,
            };

            unsafe {
                listxattr(
                    c_path.as_ptr(),
                    buf.as_mut_ptr().cast(),
                    bufsize,
                )
            }
        }

        pub fn getxattr_first(&self, c_path: &CString, c_name: &CString) -> ssize_t {
            let getxattr = match self.follow_symlinks {
                FollowSymlinks::Yes => getxattr,
                FollowSymlinks::No  => lgetxattr,
            };

            unsafe {
                getxattr(
                    c_path.as_ptr(),
                    c_name.as_ptr().cast(),
                    ptr::null_mut(),
                    0,
                )
            }
        }

        pub fn getxattr_second(&self, c_path: &CString, c_name: &CString, buf: &mut [u8], bufsize: size_t) -> ssize_t {
            let getxattr = match self.follow_symlinks {
                FollowSymlinks::Yes => getxattr,
                FollowSymlinks::No  => lgetxattr,
            };

            unsafe {
                getxattr(
                    c_path.as_ptr(),
                    c_name.as_ptr().cast(),
                    buf.as_mut_ptr().cast::<libc::c_void>(),
                    bufsize,
                )
            }
        }
    }
}
