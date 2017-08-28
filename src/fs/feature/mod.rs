// Extended attribute support
pub mod xattr;

// Git support

#[cfg(feature="git")] pub mod git;

#[cfg(not(feature="git"))]
pub mod git {
    use std::iter::FromIterator;
    use std::path::{Path, PathBuf};

    use fs::fields;


    pub struct GitCache;

    impl FromIterator<PathBuf> for GitCache {
        fn from_iter<I: IntoIterator<Item=PathBuf>>(_iter: I) -> Self {
            GitCache
        }
    }

    impl GitCache {
        pub fn get(&self, _index: &Path) -> Option<Git> {
            panic!("Tried to query a Git cache, but Git support is disabled")
        }
    }

    pub struct Git;

    impl Git {
        pub fn status(&self, _: &Path) -> fields::Git {
            panic!("Tried to get a Git status, but Git support is disabled")
        }

        pub fn dir_status(&self, path: &Path) -> fields::Git {
            self.status(path)
        }
    }
}
