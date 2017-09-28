//! Ignoring globs in `.gitignore` files.
//!
//! This uses a cache because the file with the globs in might not be the same
//! directory that we’re listing!

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use fs::filter::IgnorePatterns;


/// An **ignore cache** holds sets of glob patterns paired with the
/// directories that they should be ignored underneath. Believe it or not,
/// that’s a valid English sentence.
#[derive(Default, Debug)]
pub struct IgnoreCache {
    entries: RwLock<Vec<(PathBuf, IgnorePatterns)>>
}

impl IgnoreCache {
    pub fn new() -> IgnoreCache {
        IgnoreCache::default()
    }

    #[allow(unused_results)]  // don’t do this
    pub fn discover_underneath(&self, path: &Path) {
        let mut path = Some(path);
        let mut entries = self.entries.write().unwrap();

        while let Some(p) = path {
            if p.components().next().is_none() { break }

            let ignore_file = p.join(".gitignore");
            if ignore_file.is_file() {
                debug!("Found a .gitignore file: {:?}", ignore_file);
                if let Ok(mut file) = File::open(ignore_file) {
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).expect("Reading gitignore failed");

                    let (patterns, mut _errors) = IgnorePatterns::parse_from_iter(contents.lines());
                    entries.push((p.into(), patterns));
                }
            }
            else {
                debug!("Found no .gitignore file at {:?}", ignore_file);
            }

            path = p.parent();
        }
    }

    pub fn is_ignored(&self, suspect: &Path) -> bool {
        let entries = self.entries.read().unwrap();
        entries.iter().any(|&(ref base_path, ref patterns)| {
            if let Ok(suffix) = suspect.strip_prefix(&base_path) {
                patterns.is_ignored_path(suffix)
            }
            else {
                false
            }
        })
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty() {
        let ignores = IgnoreCache::default();
        assert_eq!(false, ignores.is_ignored(Path::new("/usr/bin/drinking")));
        assert_eq!(false, ignores.is_ignored(Path::new("target/debug/exa")));
    }
}
