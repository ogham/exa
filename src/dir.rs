use std::io::{fs, IoResult};
use file::File;

/// A **Dir** provides a cached list of the file paths in a directory that's
/// being listed.
///
/// This object gets passed to the Files themselves, in order for them to
/// check the existence of surrounding files, then highlight themselves
/// accordingly. (See `File#get_source_files`)
pub struct Dir {
    contents: Vec<Path>,
    path: Path,
}

impl Dir {
    /// Create a new Dir object filled with all the files in the directory
    /// pointed to by the given path. Fails if the directory can't be read, or
    /// isn't actually a directory.
    pub fn readdir(path: Path) -> IoResult<Dir> {
        fs::readdir(&path).map(|paths| Dir {
            contents: paths,
            path: path.clone(),
        })
    }

    /// Produce a vector of File objects from an initialised directory,
    /// printing out an error if any of the Files fail to be created.
    pub fn files(&self) -> Vec<File> {
        let mut files = vec![];

        for path in self.contents.iter() {
            match File::from_path(path, Some(self)) {
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
}
