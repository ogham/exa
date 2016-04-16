// Extended attribute support
pub mod xattr;

// Git support

#[cfg(feature="git")] mod git;
#[cfg(feature="git")] pub use self::git::Git;

#[cfg(not(feature="git"))] pub struct Git;
#[cfg(not(feature="git"))] use std::path::Path;
#[cfg(not(feature="git"))] use file::fields;

#[cfg(not(feature="git"))]
impl Git {
    pub fn scan(_: &Path) -> Result<Git, ()> {
        Err(())
    }

    pub fn status(&self, _: &Path) -> fields::Git {
        panic!("Tried to access a Git repo without Git support!");
    }

    pub fn dir_status(&self, path: &Path) -> fields::Git {
        self.status(path)
    }
}
