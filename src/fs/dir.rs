use std::io::{self, Result as IOResult};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use scoped_threadpool::Pool;

use fs::File;
use fs::feature::ignore::IgnoreCache;


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
    pub fn read_dir(path: PathBuf) -> IOResult<Dir> {
        info!("Reading directory {:?}", &path);

        let contents = fs::read_dir(&path)?
                                             .map(|result| result.map(|entry| entry.path()))
                                             .collect::<Result<_,_>>()?;

        Ok(Dir { contents, path })
    }

    /// Produce an iterator of IO results of trying to read all the files in
    /// this directory.
    pub fn files<'dir, 'ig>(&'dir self, dots: DotFilter, ignore: Option<&'ig IgnoreCache>, pool: &mut Pool) -> Files<'dir, 'ig> {
        if let Some(i) = ignore { i.discover_underneath(&self.path); }

        // File::new calls std::fs::File::metadata, which on linux calls lstat. On some
        // filesystems this can be very slow, but there's no async filesystem API
        // so all we can do to hide the latency is use system threads.
        let mut metadata = vec![None; self.contents.len()];
        let chunksize = (self.contents.len() / pool.thread_count() as usize).max(1);
        pool.scoped(|scoped| {
            for (path_chunk, meta_chunk) in self.contents.chunks(chunksize).zip(metadata.chunks_mut(chunksize)) {
                scoped.execute(move || {
                    for (path, e) in path_chunk.iter().zip(meta_chunk.iter_mut()) {
                        let meta = fs::symlink_metadata(path).map_err(Arc::new);
                        let link_target = if meta.as_ref().map(|m| m.file_type().is_symlink()).unwrap_or(false) {
                            Some(fs::metadata(path).map_err(Arc::new))
                        } else {
                            None
                        };
                        *e = Some((meta, link_target));
                    }
                });
            }
        });
        let metadata = metadata.into_iter().map(|m| m.unwrap()).collect::<Vec<_>>();

        Files {
            inner:     self.contents.iter().zip(metadata),
            dir:       self,
            dotfiles:  dots.shows_dotfiles(),
            dots:      dots.dots(),
            ignore,
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
    inner: std::iter::Zip<std::slice::Iter<'dir, PathBuf>, std::vec::IntoIter<
        (Result<fs::Metadata, Arc<io::Error>>, Option<Result<fs::Metadata, Arc<io::Error>>>)>>,

    /// The directory that begat those paths.
    dir: &'dir Dir,

    /// Whether to include dotfiles in the list.
    dotfiles: bool,

    /// Whether the `.` or `..` directories should be produced first, before
    /// any files have been listed.
    dots: Dots,

    ignore: Option<&'ig IgnoreCache>,
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
            if let Some((path, (metadata, target_metadata))) = self.inner.next() {
                let filename = File::filename(&path);
                if !self.dotfiles && filename.starts_with('.') { continue }

                if let Some(i) = self.ignore {
                    if i.is_ignored(&path) { continue }
                }

                let target_metadata = target_metadata.map(|m| m.map_err(|e| Arc::try_unwrap(e).unwrap()));

                return Some(metadata.map(|meta|
                    File {
                        name: filename,
                        ext: File::ext(path),
                        path: path.to_path_buf(),
                        metadata: meta,
                        parent_dir: Some(self.dir),
                        target_metadata,
                    }
                ).map_err(|e| {
                    (path.clone(), Arc::try_unwrap(e).unwrap())
                }))
            }
            else {
                return None
            }
        }
    }
}

/// The dot directories that need to be listed before actual files, if any.
/// If these aren’t being printed, then `FilesNext` is used to skip them.
enum Dots {

    /// List the `.` directory next.
    DotNext,

    /// List the `..` directory next.
    DotDotNext,

    /// Forget about the dot directories and just list files.
    FilesNext,
}


impl<'dir, 'ig> Iterator for Files<'dir, 'ig> {
    type Item = Result<File<'dir>, (PathBuf, io::Error)>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Dots::DotNext = self.dots {
            self.dots = Dots::DotDotNext;
            Some(File::new(self.dir.path.to_path_buf(), self.dir, String::from("."))
                      .map_err(|e| (Path::new(".").to_path_buf(), e)))
        }
        else if let Dots::DotDotNext = self.dots {
            self.dots = Dots::FilesNext;
            Some(File::new(self.parent(), self.dir, String::from(".."))
                      .map_err(|e| (self.parent(), e)))
        }
        else {
            self.next_visible_file()
        }
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

impl DotFilter {

    /// Whether this filter should show dotfiles in a listing.
    fn shows_dotfiles(self) -> bool {
        match self {
            DotFilter::JustFiles       => false,
            DotFilter::Dotfiles        => true,
            DotFilter::DotfilesAndDots => true,
        }
    }

    /// Whether this filter should add dot directories to a listing.
    fn dots(self) -> Dots {
        match self {
            DotFilter::JustFiles       => Dots::FilesNext,
            DotFilter::Dotfiles        => Dots::FilesNext,
            DotFilter::DotfilesAndDots => Dots::DotNext,
        }
    }
}
