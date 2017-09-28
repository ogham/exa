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

                    match file.read_to_string(&mut contents) {
                        Ok(_) => {
                            let patterns = file_lines_to_patterns(contents.lines());
                            entries.push((p.into(), patterns));
                        }
                        Err(e) => debug!("Failed to read a .gitignore: {:?}", e)
                    }
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


fn file_lines_to_patterns<'a, I>(iter: I) -> IgnorePatterns
where I: Iterator<Item=&'a str>
{
    let iter = iter.filter(|el| !el.is_empty());
    let iter = iter.filter(|el| !el.starts_with("#"));

    // TODO: Figure out if this should trim whitespace or not

    // Errors are currently being ignored... not a good look
    IgnorePatterns::parse_from_iter(iter).0
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_nothing() {
        use std::iter::empty;
        let (patterns, _) = IgnorePatterns::parse_from_iter(empty());
        assert_eq!(patterns, file_lines_to_patterns(empty()));
    }

    #[test]
    fn parse_some_globs() {
        let stuff = vec![ "*.mp3", "README.md" ];
        let reals = vec![ "*.mp3", "README.md" ];
        let (patterns, _) = IgnorePatterns::parse_from_iter(reals.into_iter());
        assert_eq!(patterns, file_lines_to_patterns(stuff.into_iter()));
    }

    #[test]
    fn parse_some_comments() {
        let stuff = vec![ "*.mp3", "# I am a comment!", "#", "README.md" ];
        let reals = vec![ "*.mp3",                           "README.md" ];
        let (patterns, _) = IgnorePatterns::parse_from_iter(reals.into_iter());
        assert_eq!(patterns, file_lines_to_patterns(stuff.into_iter()));
    }

    #[test]
    fn parse_some_blank_lines() {
        let stuff = vec![ "*.mp3", "", "", "README.md" ];
        let reals = vec![ "*.mp3",         "README.md" ];
        let (patterns, _) = IgnorePatterns::parse_from_iter(reals.into_iter());
        assert_eq!(patterns, file_lines_to_patterns(stuff.into_iter()));
    }

    #[test]
    fn parse_some_whitespacey_lines() {
        let stuff = vec![ " *.mp3", "  ", "  a  ", "README.md   " ];
        let reals = vec![ " *.mp3", "  ", "  a  ", "README.md   " ];
        let (patterns, _) = IgnorePatterns::parse_from_iter(reals.into_iter());
        assert_eq!(patterns, file_lines_to_patterns(stuff.into_iter()));
    }



    #[test]
    fn an_empty_cache_ignores_nothing() {
        let ignores = IgnoreCache::default();
        assert_eq!(false, ignores.is_ignored(Path::new("/usr/bin/drinking")));
        assert_eq!(false, ignores.is_ignored(Path::new("target/debug/exa")));
    }
}
