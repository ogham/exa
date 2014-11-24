use std::io::{fs, IoResult};
use file::File;

// The purpose of a Dir is to provide a cached list of the file paths
// in the directory being searched for. This object is then passed to
// the Files themselves, which can then check the status of their
// surrounding files, such as whether it needs to be coloured
// differently if a certain other file exists.

pub struct Dir<'a> {
    pub contents: Vec<Path>,
    pub path: Path,
}

impl<'a> Dir<'a> {
    pub fn readdir(path: Path) -> IoResult<Dir<'a>> {
        fs::readdir(&path).map(|paths| Dir {
            contents: paths,
            path: path.clone(),
        })
    }

    pub fn files(&'a self) -> Vec<File<'a>> {
        let mut files = vec![];
        
        for path in self.contents.iter() {
            match File::from_path(path.clone(), Some(self)) {
                Ok(file) => files.push(file),
                Err(e)   => println!("{}: {}", path.display(), e),
            }
        }
        
        files
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.contents.contains(path)
    }
}


