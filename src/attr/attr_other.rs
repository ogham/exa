//! Extended attribute support for other os
use std::old_io as io;

/// Extended attribute
#[derive(Clone)]
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
pub fn list(_: &Path) -> io::IoResult<Vec<Attribute>> {
    Ok(Vec::new())
}
/// Lists the extended attributes. Does not follow symlinks like `lstat`
pub fn llist(_: &Path) -> io::IoResult<Vec<Attribute>> {
    Ok(Vec::new())
}

/// Returns true if the extended attribute feature is implemented on this platform.
#[inline(always)]
pub fn feature_implemented() -> bool { false }