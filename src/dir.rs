use std::io;
use std::fs;
use std::path::{Path, PathBuf};
use std::slice::Iter as SliceIter;

use feature::Git;
use file::{File, fields};


/// A **Dir** provides a cached list of the file paths in a directory that's
/// being listed.
///
/// This object gets passed to the Files themselves, in order for them to
/// check the existence of surrounding files, then highlight themselves
/// accordingly. (See `File#get_source_files`)
pub struct Dir {

    /// A vector of the files that have been read from this directory.
    contents: Vec<PathBuf>,

    /// The path that was read.
    pub path: PathBuf,

    /// Holds a `Git` object if scanning for Git repositories is switched on,
    /// and this directory happens to contain one.
    git: Option<Git>,
}

impl Dir {

    /// Create a new Dir object filled with all the files in the directory
    /// pointed to by the given path. Fails if the directory can't be read, or
    /// isn't actually a directory, or if there's an IO error that occurs
    /// while scanning.
    pub fn read_dir(path: &Path, git: bool) -> io::Result<Dir> {
        let reader = try!(fs::read_dir(path));
        let contents = try!(reader.map(|e| e.map(|e| e.path())).collect());

        Ok(Dir {
            contents: contents,
            path: path.to_path_buf(),
            git: if git { Git::scan(path).ok() } else { None },
        })
    }

    /// Produce an iterator of IO results of trying to read all the files in
    /// this directory.
    pub fn files<'dir>(&'dir self) -> Files<'dir> {
        Files {
            inner: self.contents.iter(),
            dir: &self,
        }
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
    pub fn git_status(&self, path: &Path, prefix_lookup: bool) -> fields::Git {
        match (&self.git, prefix_lookup) {
            (&Some(ref git), false)  => git.status(path),
            (&Some(ref git), true)   => git.dir_status(path),
            (&None, _)               => fields::Git::empty()
        }
    }
}


/// Iterator over reading the contents of a directory as `File` objects.
pub struct Files<'dir> {
    inner: SliceIter<'dir, PathBuf>,
    dir: &'dir Dir,
}

impl<'dir> Iterator for Files<'dir> {
    type Item = Result<File<'dir>, (PathBuf, io::Error)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|path| File::from_path(path, Some(self.dir)).map_err(|t| (path.clone(), t)))
    }
}