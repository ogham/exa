//! Extended attribute support for darwin
extern crate libc;

use std::ffi::CString;
use std::ptr;
use std::old_io as io;
use std::old_path::GenericPath;
use std::old_path::posix::Path;
use self::libc::{size_t, ssize_t, c_char, c_void};

extern "C" {
    fn listxattr(path: *const c_char, list: *mut c_char, size: size_t) -> ssize_t;
    fn llistxattr(path: *const c_char, list: *mut c_char, size: size_t) -> ssize_t;
    fn getxattr(path: *const c_char, name: *const c_char,
                value: *mut c_void, size: size_t
    ) -> ssize_t;
    fn lgetxattr(path: *const c_char, name: *const c_char,
                value: *mut c_void, size: size_t
    ) -> ssize_t;
}

/// Attributes which can be passed to `Attribute::list_with_flags`
#[derive(Copy)]
pub enum FollowSymlinks {
    Yes,
    No
}

/// Extended attribute
#[derive(Debug, Clone)]
pub struct Attribute {
    name: String,
    size: usize,
}

impl Attribute {
    /// Lists the extended attribute of `path`.
    /// Does follow symlinks by default.
    pub fn list(path: &Path, do_follow: FollowSymlinks) -> io::IoResult<Vec<Attribute>> {
        let (listxattr, getxattr) = match do_follow {
            FollowSymlinks::Yes => (listxattr, getxattr),
            FollowSymlinks::No => (llistxattr, lgetxattr),
        };
        let c_path = try!(CString::new(path.as_vec()));
        let bufsize = unsafe {
            listxattr(c_path.as_ptr(), ptr::null_mut(), 0)
        };
        if bufsize > 0 {
            let mut buf = vec![0u8; bufsize as usize];
            let err = unsafe { listxattr(
                c_path.as_ptr(),
                buf.as_mut_ptr() as *mut c_char,
                bufsize as size_t
            )};
            if err > 0 {
                // End indicies of the attribute names
                // the buffer contains 0-terminates c-strings
                let idx = buf.iter().enumerate().filter_map(|(i, v)|
                    if *v == 0 { Some(i) } else { None }
                );
                let mut names = Vec::new();
                let mut start = 0;
                for end in idx {
                    let c_end = end + 1; // end of the c-string (including 0)
                    let size = unsafe {
                        getxattr(
                            c_path.as_ptr(),
                            buf[start..c_end].as_ptr() as *const c_char,
                            ptr::null_mut(), 0
                        )
                    };
                    if size > 0 {
                        names.push(Attribute {
                            name: String::from_utf8_lossy(&buf[start..end]).into_owned(),
                            size: size as usize
                        });
                    }
                    start = c_end;
                }
                Ok(names)
            } else {
                Err(io::IoError {
                    kind: io::OtherIoError,
                    desc: "could not read extended attributes",
                    detail: None
                })
            }
        } else {
            Err(io::IoError {
                kind: io::OtherIoError,
                desc: "could not read extended attributes",
                detail: None
            })
        }
    }

    /// Getter for name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Getter for size
    pub fn size(&self) -> usize {
        self.size
    }
}

/// Lists the extended attributes.
/// Follows symlinks like `stat`
pub fn list(path: &Path) -> io::IoResult<Vec<Attribute>> {
    Attribute::list(path, FollowSymlinks::Yes)
}
/// Lists the extended attributes.
/// Does not follow symlinks like `lstat`
pub fn llist(path: &Path) -> io::IoResult<Vec<Attribute>> {
    Attribute::list(path, FollowSymlinks::No)
}

/// Returns true if the extended attribute feature is implemented on this platform.
#[inline(always)]
pub fn feature_implemented() -> bool { true }
