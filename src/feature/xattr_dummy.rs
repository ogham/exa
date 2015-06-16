use std::io;
use std::path::Path;

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

    /// Lists the extended attributes. Follows symlinks like `metadata`
    pub fn list(_: &Path) -> io::Result<Vec<Attribute>> {
        Ok(Vec::new())
    }

    /// Lists the extended attributes. Does not follow symlinks like `symlink_metadata`
    pub fn llist(_: &Path) -> io::Result<Vec<Attribute>> {
        Ok(Vec::new())
    }

    pub fn feature_implemented() -> bool { false }
}


