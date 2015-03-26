// Extended attribute support

#[cfg(target_os = "macos")] mod xattr_darwin;
#[cfg(target_os = "macos")] pub use self::xattr_darwin::Attribute;

#[cfg(target_os = "linux")] mod xattr_linux;
#[cfg(target_os = "linux")] pub use self::xattr_linux::Attribute;

#[cfg(not(any(target_os = "macos", target_os = "linux")))] use std::old_io as io;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
#[derive(Clone)]
pub struct Attribute;

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
impl Attribute {

    /// Getter for name
    pub fn name(&self) -> &str {
        unimplemented!()
    }

    /// Getter for size
    pub fn size(&self) -> usize {
        unimplemented!()
    }

    /// Lists the extended attributes. Follows symlinks like `stat`
    pub fn list(_: &Path) -> io::IoResult<Vec<Attribute>> {
        Ok(Vec::new())
    }

    /// Lists the extended attributes. Does not follow symlinks like `lstat`
    pub fn llist(_: &Path) -> io::IoResult<Vec<Attribute>> {
        Ok(Vec::new())
    }

    pub fn feature_implemented() -> bool { false }
}



// Git support

#[cfg(feature="git")] mod git;
#[cfg(feature="git")] pub use self::git::Git;

#[cfg(not(feature="git"))] pub struct Git;
#[cfg(not(feature="git"))] use std::old_path::posix::Path;
#[cfg(not(feature="git"))]
impl Git {
    pub fn scan(_: &Path) -> Result<Git, ()> {
        Err(())
    }

    pub fn status(&self, _: &Path) -> String {
        panic!("Tried to access a Git repo without Git support!");
    }

    pub fn dir_status(&self, path: &Path) -> String {
        self.status(path)
    }
}
