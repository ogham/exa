use std::old_io::{fs, IoResult};
use std::old_path::GenericPath;
use std::old_path::posix::Path;

use feature::Git;
use file::{File, GREY};

/// A **Dir** provides a cached list of the file paths in a directory that's
/// being listed.
///
/// This object gets passed to the Files themselves, in order for them to
/// check the existence of surrounding files, then highlight themselves
/// accordingly. (See `File#get_source_files`)
pub struct Dir {
    contents: Vec<Path>,
    path: Path,
    git: Option<Git>,
}

impl Dir {

    /// Create a new Dir object filled with all the files in the directory
    /// pointed to by the given path. Fails if the directory can't be read, or
    /// isn't actually a directory.
    pub fn readdir(path: &Path) -> IoResult<Dir> {
        fs::readdir(path).map(|paths| Dir {
            contents: paths,
            path: path.clone(),
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
        self.contents.contains(path)
    }

    /// Append a path onto the path specified by this directory.
    pub fn join(&self, child: Path) -> Path {
        self.path.join(child)
    }

    /// Return whether there's a Git repository on or above this directory.
    pub fn has_git_repo(&self) -> bool {
        self.git.is_some()
    }

    /// Get a string describing the Git status of the given file.
    pub fn git_status(&self, path: &Path, prefix_lookup: bool) -> String {
        match (&self.git, prefix_lookup) {
            (&Some(ref git), false)  => git.status(path),
            (&Some(ref git), true)   => git.dir_status(path),
            (&None, _)               => GREY.paint("--").to_string(),
        }
    }
}
