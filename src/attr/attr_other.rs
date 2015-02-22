//! Extended attribute support for other os
use std::old_io as io;

/// Extended attribute
pub struct Attribute;

impl Attribute {
    
    /// Getter for name
    pub fn name(&self) -> &str {
        unimplemented!()
    }

    /// Getter for size
    pub fn size(&self) -> usize {
        unimplemented!()
    }
}

/// Lists the extended attributes. Follows symlinks like `stat`
pub fn list(path: &Path) -> io::IoResult<Vec<Attribute>> {
    Vec::new()
}
/// Lists the extended attributes. Does not follow symlinks like `lstat`
pub fn llist(path: &Path) -> io::IoResult<Vec<Attribute>> {
    Vec::new()
}

/// Returns true if the extended attribute feature is implemented on this platform.
#[inline(always)]
pub fn feature_implemented() -> bool { false }