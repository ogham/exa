use std::io::{self, Result as IOResult};
use std::fs;
use std::path::{Path, PathBuf};
use std::slice::Iter as SliceIter;

use fs::feature::Git;
use fs::{File, fields};


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
    /// pointed to by the given path. Fails if the directory can’t be read, or
    /// isn’t actually a directory, or if there’s an IO error that occurs at
    /// any point.
    ///
    /// The `read_dir` iterator doesn’t actually yield the `.` and `..`
    /// entries, so if the user wants to see them, we’ll have to add them
    /// ourselves after the files have been read.
    pub fn read_dir(path: PathBuf, dots: DotFilter, git: bool) -> IOResult<Dir> {
        let mut contents: Vec<PathBuf> = try!(fs::read_dir(&path)?
                                                 .map(|result| result.map(|entry| entry.path()))
                                                 .collect());
        match dots {
            DotFilter::JustFiles => contents.retain(|p| p.file_name().and_then(|name| name.to_str()).map(|s| !s.starts_with('.')).unwrap_or(true)),
            DotFilter::Dotfiles => {/* Don’t add or remove anything */},
            DotFilter::DotfilesAndDots => {
                contents.insert(0, path.join(".."));
                contents.insert(0, path.join("."));
            }
        }

        let git = if git { Git::scan(&path).ok() } else { None };
        Ok(Dir { contents, path, git })
    }

    /// Produce an iterator of IO results of trying to read all the files in
    /// this directory.
    pub fn files(&self) -> Files {
        Files {
            inner: self.contents.iter(),
            dir: self,
        }
    }

    /// Whether this directory contains a file with the given path.
    pub fn contains(&self, path: &Path) -> bool {
        self.contents.iter().any(|p| p.as_path() == path)
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


/// Usually files in Unix use a leading dot to be hidden or visible, but two
/// entries in particular are "extra-hidden": `.` and `..`, which only become
/// visible after an extra `-a` option.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum DotFilter {

    /// Shows files, dotfiles, and `.` and `..`.
    DotfilesAndDots,

    /// Show files and dotfiles, but hide `.` and `..`.
    Dotfiles,

    /// Just show files, hiding anything beginning with a dot.
    JustFiles,
}

impl Default for DotFilter {
    fn default() -> DotFilter {
        DotFilter::JustFiles
    }
}
