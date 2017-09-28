//! Ignoring globs in `.gitignore` files.
//!
//! This uses a cache because the file with the globs in might not be the same
//! directory that we’re listing!

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use fs::filter::IgnorePatterns;


/// An **ignore cache** holds sets of glob patterns paired with the
/// directories that they should be ignored underneath.
#[derive(Default, Debug)]
pub struct IgnoreCache {
    entries: Vec<(PathBuf, IgnorePatterns)>
}

impl IgnoreCache {
    pub fn new() -> IgnoreCache {
        IgnoreCache::default()
    }

    #[allow(unused_results)]  // don’t do this
    pub fn discover_underneath(&mut self, path: &Path) {
        let mut path = Some(path);

        while let Some(p) = path {
            let ignore_file = p.join(".gitignore");
            if ignore_file.is_file() {
                if let Ok(mut file) = File::open(ignore_file) {
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).expect("Reading gitignore failed");

                    let (patterns, mut _errors) = IgnorePatterns::parse_from_iter(contents.lines());
                    self.entries.push((p.into(), patterns));
                }
            }

            path = p.parent();
        }
    }

    pub fn is_ignored(&self, suspect: &Path) -> bool {
        self.entries.iter().any(|&(ref base_path, ref patterns)| {
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
