pub mod ignore;
pub mod xattr;

#[cfg(feature = "git")]
pub mod git;

#[cfg(not(feature = "git"))]
pub mod git {
    use std::iter::FromIterator;
    use std::path::{Path, PathBuf};

    use fs::fields as f;

    pub struct GitCache;

    impl FromIterator<PathBuf> for GitCache {
        fn from_iter<I: IntoIterator<Item = PathBuf>>(_iter: I) -> Self {
            GitCache
        }
    }

    impl GitCache {
        pub fn has_anything_for(&self, _index: &Path) -> bool {
            false
        }

        pub fn get(&self, _index: &Path, _prefix_lookup: bool) -> f::Git {
            panic!("Tried to query a Git cache, but Git support is disabled")
        }
    }
}
