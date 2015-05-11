use colours::Colours;
use feature::Git;
use file::File;
use file;

use std::io;
use std::fs;
use std::path::{Path, PathBuf};

/// A **Dir** provides a cached list of the file paths in a directory that's
/// being listed.
///
/// This object gets passed to the Files themselves, in order for them to
/// check the existence of surrounding files, then highlight themselves
/// accordingly. (See `File#get_source_files`)
pub struct Dir {
    contents: Vec<PathBuf>,
    path: PathBuf,
    git: Option<Git>,
}

impl Dir {

    /// Create a new Dir object filled with all the files in the directory
    /// pointed to by the given path. Fails if the directory can't be read, or
    /// isn't actually a directory.
    pub fn readdir(path: &Path) -> io::Result<Dir> {
        fs::read_dir(path).map(|dir_obj| Dir {
            contents: dir_obj.map(|entry| entry.unwrap().path()).collect(),
            path: path.to_path_buf(),
            git: Git::scan(path).ok(),
        })
    }

    /// Produce a vector of File objects from an initialised directory,
    /// printing out an error if any of the Files fail to be created.
    ///
    /// Passing in `recurse` means that any directories will be scanned for
    /// their contents, as well.
    pub fn files(&self, recurse: bool) -> Vec<File> {
        let mut files = vec![];

        for path in self.contents.iter() {
            match File::from_path(path, Some(self), recurse) {
                Ok(file) => files.push(file),
                Err(e)   => println!("{}: {}", path.display(), e),
            }
        }

        files
    }

    /// Whether this directory contains a file with the given path.
    pub fn contains(&self, path: &Path) -> bool {
        self.contents.iter().any(|ref p| p.as_path() == path)
    }

    /// Append a path onto the path specified by this directory.
    pub fn join(&self, child: &Path) -> PathBuf {
        self.path.join(child)
    }

    /// Return whether there's a Git repository on or above this directory.
    pub fn has_git_repo(&self) -> bool {
        self.git.is_some()
    }

    /// Get a string describing the Git status of the given file.
    pub fn git_status(&self, path: &Path, prefix_lookup: bool) -> file::Git {
        match (&self.git, prefix_lookup) {
            (&Some(ref git), false)  => git.status(path),
            (&Some(ref git), true)   => git.dir_status(path),
            (&None, _)               => file::Git::empty()
        }
    }
}
