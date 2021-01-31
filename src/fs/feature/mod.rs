pub mod xattr;

#[cfg(feature = "git")]
pub mod git;

#[cfg(not(feature = "git"))]
pub mod git {
    use std::iter::FromIterator;
    use std::path::{Path, PathBuf};

    use crate::fs::fields as f;


    pub struct GitCache;

    impl FromIterator<PathBuf> for GitCache {
        fn from_iter<I>(_iter: I) -> Self
        where I: IntoIterator<Item=PathBuf>
        {
            Self
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

    impl f::SubdirGitRepo{
        pub fn from_path(_dir : &Path) -> Self{
            Self{status : f::SubdirGitRepoStatus::NotRepo}
        }
    }
}
