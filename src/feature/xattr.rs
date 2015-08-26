//! Extended attribute support for Darwin and Linux systems.
extern crate libc;

use std::io;
use std::path::Path;


pub const ENABLED: bool = cfg!(feature="git") && cfg!(any(target_os="macos", target_os="linux"));

pub trait FileAttributes {
    fn attributes(&self) -> io::Result<Vec<Attribute>>;
    fn symlink_attributes(&self) -> io::Result<Vec<Attribute>>;
}

impl FileAttributes for Path {
    fn attributes(&self) -> io::Result<Vec<Attribute>> {
        list_attrs(lister::Lister::new(FollowSymlinks::Yes), &self)
    }

    fn symlink_attributes(&self) -> io::Result<Vec<Attribute>> {
        list_attrs(lister::Lister::new(FollowSymlinks::No), &self)
    }
}

/// Attributes which can be passed to `Attribute::list_with_flags`
#[derive(Copy, Clone)]
pub enum FollowSymlinks {
    Yes,
    No
}

/// Extended attribute
#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub size: usize,
}

pub fn list_attrs(lister: lister::Lister, path: &Path) -> io::Result<Vec<Attribute>> {
    let c_path = match path.as_os_str().to_cstring() {
        Some(cstring) => cstring,
        None => return Err(io::Error::new(io::ErrorKind::Other, "Error: path somehow contained a NUL?")),
    };

    let mut names = Vec::new();
    let bufsize = lister.listxattr_first(&c_path);

    if bufsize < 0 {
        return Err(io::Error::last_os_error());
    }
    else if bufsize > 0 {
        let mut buf = vec![0u8; bufsize as usize];
        let err = lister.listxattr_second(&c_path, &mut buf, bufsize);

        if err < 0 {
            return Err(io::Error::last_os_error());
        }

        if err > 0 {
            // End indicies of the attribute names
            // the buffer contains 0-terminates c-strings
            let idx = buf.iter().enumerate().filter_map(|(i, v)|
                if *v == 0 { Some(i) } else { None }
            );
            let mut start = 0;

            for end in idx {
                let c_end = end + 1; // end of the c-string (including 0)
                let size = lister.getxattr(&c_path, &buf[start..c_end]);

                if size > 0 {
                    names.push(Attribute {
                        name: lister.translate_attribute_name(&buf[start..end]),
                        size: size as usize
                    });
                }

                start = c_end;
            }

        }

    }
    Ok(names)
}

#[cfg(target_os = "macos")]
mod lister {
    use std::ffi::CString;
    use libc::{c_int, size_t, ssize_t, c_char, c_void, uint32_t};
    use super::FollowSymlinks;
    use std::ptr;

    extern "C" {
        fn listxattr(
            path: *const c_char, namebuf: *mut c_char,
            size: size_t, options: c_int
        ) -> ssize_t;

        fn getxattr(
            path: *const c_char, name: *const c_char,
            value: *mut c_void, size: size_t, position: uint32_t,
            options: c_int
        ) -> ssize_t;
    }

    pub struct Lister {
        c_flags: c_int,
    }

    impl Lister {
        pub fn new(do_follow: FollowSymlinks) -> Lister {
            let c_flags: c_int = match do_follow {
                FollowSymlinks::Yes => 0x0001,
                FollowSymlinks::No  => 0x0000,
            };

            Lister { c_flags: c_flags }
        }

        pub fn translate_attribute_name(&self, input: &[u8]) -> String {
            use std::str::from_utf8_unchecked;

            unsafe {
                from_utf8_unchecked(input).into()
            }
        }

        pub fn listxattr_first(&self, c_path: &CString) -> ssize_t {
            unsafe {
                listxattr(c_path.as_ptr(), ptr::null_mut(), 0, self.c_flags)
            }
        }

        pub fn listxattr_second(&self, c_path: &CString, buf: &mut Vec<u8>, bufsize: ssize_t) -> ssize_t {
            unsafe {
                listxattr(
                    c_path.as_ptr(),
                    buf.as_mut_ptr() as *mut c_char,
                    bufsize as size_t, self.c_flags
                )
            }
        }

        pub fn getxattr(&self, c_path: &CString, buf: &[u8]) -> ssize_t {
            unsafe {
                getxattr(
                    c_path.as_ptr(),
                    buf.as_ptr() as *const c_char,
                    ptr::null_mut(), 0, 0, self.c_flags
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
            path: *const c_char, list: *mut c_char, size: size_t
        ) -> ssize_t;

        fn llistxattr(
            path: *const c_char, list: *mut c_char, size: size_t
        ) -> ssize_t;

        fn getxattr(
            path: *const c_char, name: *const c_char,
            value: *mut c_void, size: size_t
        ) -> ssize_t;

        fn lgetxattr(
            path: *const c_char, name: *const c_char,
            value: *mut c_void, size: size_t
        ) -> ssize_t;
    }

    pub struct Lister {
        follow_symlinks: FollowSymlinks,
    }

    impl Lister {
        pub fn new(follow_symlinks: FollowSymlinks) -> Lister {
            Lister { follow_symlinks: follow_symlinks }
        }

        pub fn translate_attribute_name(&self, input: &[u8]) -> String {
            String::from_utf8_lossy(input).into_owned()
        }

        pub fn listxattr_first(&self, c_path: &CString) -> ssize_t {
            let listxattr = match self.follow_symlinks {
                FollowSymlinks::Yes => listxattr,
                FollowSymlinks::No  => llistxattr,
            };

            unsafe {
                listxattr(c_path.as_ptr(), ptr::null_mut(), 0)
            }
        }

        pub fn listxattr_second(&self, c_path: &CString, buf: &mut Vec<u8>, bufsize: ssize_t) -> ssize_t {
            let listxattr = match self.follow_symlinks {
                FollowSymlinks::Yes => listxattr,
                FollowSymlinks::No  => llistxattr,
            };

            unsafe {
                listxattr(
                    c_path.as_ptr(),
                    buf.as_mut_ptr() as *mut c_char,
                    bufsize as size_t
                )
            }
        }

        pub fn getxattr(&self, c_path: &CString, buf: &[u8]) -> ssize_t {
            let getxattr = match self.follow_symlinks {
                FollowSymlinks::Yes => getxattr,
                FollowSymlinks::No  => lgetxattr,
            };

            unsafe {
                getxattr(
                    c_path.as_ptr(),
                    buf.as_ptr() as *const c_char,
                    ptr::null_mut(), 0
                )
            }
        }
    }
}