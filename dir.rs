use std::io::fs; 
use file::File;

// The purpose of a Dir is to provide a cached list of the file paths
// in the directory being searched for. This object is then passed to
// the Files themselves, which can then check the status of their
// surrounding files, such as whether it needs to be coloured
// differently if a certain other file exists.

pub struct Dir<'a> {
    contents: Vec<Path>,
}

impl<'a> Dir<'a> {
    pub fn readdir(path: Path) -> Dir<'a> {
        match fs::readdir(&path) {
            Ok(paths) => Dir {
                contents: paths,
            },
            Err(e) => fail!("readdir: {}", e),
        }
    }

    pub fn files(&'a self) -> Vec<File<'a>> {
        self.contents.iter().map(|path| File::from_path(path, self)).collect()
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.contents.contains(path)
    }
}


