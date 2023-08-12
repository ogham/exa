use crate::fs::feature::git::GitCache;
use crate::fs::fields::GitStatus;
use std::io;
use std::fs;
use std::path::{Path, PathBuf};
use std::slice::Iter as SliceIter;

use log::*;

use crate::fs::File;


/// A **Dir** provides a cached list of the file paths in a directory that’s
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
    pub fn read_dir(path: PathBuf) -> io::Result<Self> {
        info!("Reading directory {:?}", &path);

        let contents = fs::read_dir(&path)?
                          .map(|result| result.map(|entry| entry.path()))
                          .collect::<Result<_, _>>()?;

        Ok(Self { contents, path })
    }

    /// Produce an iterator of IO results of trying to read all the files in
    /// this directory.
    pub fn files<'dir, 'ig>(&'dir self, dots: DotFilter, git: Option<&'ig GitCache>, git_ignoring: bool) -> Files<'dir, 'ig> {
        Files {
            inner:     self.contents.iter(),
            dir:       self,
            dotfiles:  dots.shows_dotfiles(),
            dots:      dots.dots(),
            git,
            git_ignoring,
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
}


/// Iterator over reading the contents of a directory as `File` objects.
pub struct Files<'dir, 'ig> {

    /// The internal iterator over the paths that have been read already.
    inner: SliceIter<'dir, PathBuf>,

    /// The directory that begat those paths.
    dir: &'dir Dir,

    /// Whether to include dotfiles in the list.
    dotfiles: bool,

    /// Whether the `.` or `..` directories should be produced first, before
    /// any files have been listed.
    dots: DotsNext,

    git: Option<&'ig GitCache>,

    git_ignoring: bool,
}

impl<'dir, 'ig> Files<'dir, 'ig> {
    fn parent(&self) -> PathBuf {
        // We can’t use `Path#parent` here because all it does is remove the
        // last path component, which is no good for us if the path is
        // relative. For example, while the parent of `/testcases/files` is
        // `/testcases`, the parent of `.` is an empty path. Adding `..` on
        // the end is the only way to get to the *actual* parent directory.
        self.dir.path.join("..")
    }

    /// Go through the directory until we encounter a file we can list (which
    /// varies depending on the dotfile visibility flag)
    fn next_visible_file(&mut self) -> Option<Result<File<'dir>, (PathBuf, io::Error)>> {
        loop {
            if let Some(path) = self.inner.next() {
                let filename = File::filename(path);
                if ! self.dotfiles && filename.starts_with('.') {
                    continue;
                }

                // Also hide _prefix files on Windows because it's used by old applications
                // as an alternative to dot-prefix files.
                #[cfg(windows)]
                if ! self.dotfiles && filename.starts_with('_') {
                    continue;
                }

                if self.git_ignoring {
                    let git_status = self.git.map(|g| g.get(path, false)).unwrap_or_default();
                    if git_status.unstaged == GitStatus::Ignored {
                         continue;
                    }
                }

                return Some(File::from_args(path.clone(), self.dir, filename)
                                 .map_err(|e| (path.clone(), e)))
            }

            return None
        }
    }
}

/// The dot directories that need to be listed before actual files, if any.
/// If these aren’t being printed, then `FilesNext` is used to skip them.
enum DotsNext {

    /// List the `.` directory next.
    Dot,

    /// List the `..` directory next.
    DotDot,

    /// Forget about the dot directories and just list files.
    Files,
}

impl<'dir, 'ig> Iterator for Files<'dir, 'ig> {
    type Item = Result<File<'dir>, (PathBuf, io::Error)>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.dots {
            DotsNext::Dot => {
                self.dots = DotsNext::DotDot;
                Some(File::new_aa_current(self.dir)
                          .map_err(|e| (Path::new(".").to_path_buf(), e)))
            }

            DotsNext::DotDot => {
                self.dots = DotsNext::Files;
                Some(File::new_aa_parent(self.parent(), self.dir)
                          .map_err(|e| (self.parent(), e)))
            }

            DotsNext::Files => {
                self.next_visible_file()
            }
        }
    }
}


/// Usually files in Unix use a leading dot to be hidden or visible, but two
/// entries in particular are “extra-hidden”: `.` and `..`, which only become
/// visible after an extra `-a` option.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum DotFilter {

    /// Shows files, dotfiles, and `.` and `..`.
    DotfilesAndDots,

    /// Show files and dotfiles, but hide `.` and `..`.
    Dotfiles,

    /// Just show files, hiding anything beginning with a dot.
    JustFiles,
}

impl Default for DotFilter {
    fn default() -> Self {
        Self::JustFiles
    }
}

impl DotFilter {

    /// Whether this filter should show dotfiles in a listing.
    fn shows_dotfiles(self) -> bool {
        match self {
            Self::JustFiles       => false,
            Self::Dotfiles        => true,
            Self::DotfilesAndDots => true,
        }
    }

    /// Whether this filter should add dot directories to a listing.
    fn dots(self) -> DotsNext {
        match self {
            Self::JustFiles        => DotsNext::Files,
            Self::Dotfiles         => DotsNext::Files,
            Self::DotfilesAndDots  => DotsNext::Dot,
        }
    }
}
