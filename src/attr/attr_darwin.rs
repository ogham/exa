//! Extended attribute support for darwin
extern crate libc;

use std::ffi::CString;
use std::ptr;
use std::mem;
use std::old_io as io;
use self::libc::{c_int, size_t, ssize_t, c_char, c_void, uint32_t};

/// Don't follow symbolic links
const XATTR_NOFOLLOW: c_int = 0x0001; 
/// Expose HFS Compression extended attributes
const XATTR_SHOWCOMPRESSION: c_int = 0x0020; 

extern "C" {
    fn listxattr(path: *const c_char, namebuf: *mut c_char,
                 size: size_t, options: c_int) -> ssize_t;
    fn getxattr(path: *const c_char, name: *const c_char,
                value: *mut c_void, size: size_t, position: uint32_t,
                options: c_int) -> ssize_t;
}

/// Attributes which can be passed to `Attribute::list_with_flags`
#[derive(Copy)]
pub enum ListFlags {
    /// Don't follow symbolic links
    NoFollow = XATTR_NOFOLLOW as isize,
    /// Expose HFS Compression extended attributes
    ShowCompression = XATTR_SHOWCOMPRESSION as isize
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
    pub fn list(path: &Path, flags: &[ListFlags]) -> io::IoResult<Vec<Attribute>> {
        let mut c_flags: c_int = 0;
        for &flag in flags.iter() {
            c_flags |= flag as c_int
        }
        let c_path = try!(CString::new(path.as_vec()));
        let bufsize = unsafe { listxattr(
            c_path.as_ptr(),
            ptr::null_mut(),
            0, c_flags
        )} as isize;
        if bufsize > 0 {
            let mut buf = vec![0u8; bufsize as usize];
            let err = unsafe { listxattr(
                c_path.as_ptr(),
                buf.as_mut_ptr() as *mut c_char,
                bufsize as size_t, c_flags
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
                    let size = unsafe {
                        getxattr(
                            c_path.as_ptr(),
                            buf[start..end+1].as_ptr() as *const c_char,
                            ptr::null_mut(), 0, 0, c_flags
                        )
                    };
                    if size > 0 {
                        names.push(Attribute { 
                            name: unsafe {
                                // buf is guaranteed to contain valid utf8 strings
                                // see man listxattr
                                mem::transmute::<&[u8], &str>(&buf[start..end]).to_string() 
                            },
                            size: size as usize
                        });
                    }
                    start = end + 1;
                }
                Ok(names)
            } else {
                // Ignore error for now
                Ok(Vec::new())
            }
        } else {
            // Ignore error for now
            Ok(Vec::new())
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
    Attribute::list(path, &[])
}
/// Lists the extended attributes.
/// Does not follow symlinks like `lstat`
pub fn llist(path: &Path) -> io::IoResult<Vec<Attribute>> {
    Attribute::list(path, &[ListFlags::NoFollow])
}

/// Returns true if the extended attribute feature is implemented on this platform.
#[inline(always)]
pub fn feature_implemented() -> bool { true }