// Extended attribute support

#[cfg(target_os = "macos")] mod xattr_darwin;
#[cfg(target_os = "macos")] pub use self::xattr_darwin::Attribute;

#[cfg(target_os = "linux")] mod xattr_linux;
#[cfg(target_os = "linux")] pub use self::xattr_linux::Attribute;

#[cfg(not(any(target_os = "macos", target_os = "linux")))] mod xattr_dummy;
#[cfg(not(any(target_os = "macos", target_os = "linux")))] pub use self::xattr_dummy::Attribute;

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
