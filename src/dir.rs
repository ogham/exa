use std::io::{fs, IoResult};
use file::{File, GREY};

#[cfg(feature="git")] use ansi_term::ANSIString;
#[cfg(feature="git")] use ansi_term::Colour::*;
#[cfg(feature="git")] use git2;

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
    pub fn readdir(path: Path) -> IoResult<Dir> {
        fs::readdir(&path).map(|paths| Dir {
            contents: paths,
            path: path.clone(),
            git: Git::scan(&path).ok(),
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

    /// Return whether there's a Git repository on or above this directory.
    pub fn has_git_repo(&self) -> bool {
        self.git.is_some()
    }

    /// Get a string describing the Git status of the given file.
    pub fn git_status(&self, path: &Path) -> String {
        match self.git {
            Some(ref git) => git.status(path),
            None => GREY.paint("--").to_string(),
        }
    }
}

/// Container of Git statuses for all the files in this folder's Git repository.
#[cfg(feature="git")]
struct Git {
    statuses: Vec<(String, git2::Status)>,
}

#[cfg(feature="git")]
impl Git {

    /// Discover a Git repository on or above this directory, scanning it for
    /// the files' statuses if one is found.
    fn scan(path: &Path) -> Result<Git, git2::Error> {
        let repo = try!(git2::Repository::discover(path));
        let statuses = try!(repo.statuses(None));

        Ok(Git { statuses: statuses.iter().map(|e| (e.path().unwrap().to_string(), e.status())).collect() })
    }

    /// Get the status for the file at the given path, if present.
    fn status(&self, path: &Path) -> String {
        match self.statuses.iter().find(|&&(ref p, _)| path.as_str().unwrap() == p.as_slice()) {
            Some(&(_, s)) => format!("{}{}", Git::index_status(s), Git::working_tree_status(s)),
            None => GREY.paint("--").to_string(),
        }
    }

    /// The character to display if the file has been modified, but not staged.
    fn working_tree_status(status: git2::Status) -> ANSIString<'static> {
        match status {
            s if s.contains(git2::STATUS_WT_NEW) => Green.paint("A"),
            s if s.contains(git2::STATUS_WT_MODIFIED) => Blue.paint("M"),
            s if s.contains(git2::STATUS_WT_DELETED) => Red.paint("D"),
            s if s.contains(git2::STATUS_WT_RENAMED) => Yellow.paint("R"),
            s if s.contains(git2::STATUS_WT_TYPECHANGE) => Purple.paint("T"),
            _ => GREY.paint("-"),
        }
    }

    /// The character to display if the file has been modified, and the change
    /// has been staged.
    fn index_status(status: git2::Status) -> ANSIString<'static> {
        match status {
            s if s.contains(git2::STATUS_INDEX_NEW) => Green.paint("A"),
            s if s.contains(git2::STATUS_INDEX_MODIFIED) => Blue.paint("M"),
            s if s.contains(git2::STATUS_INDEX_DELETED) => Red.paint("D"),
            s if s.contains(git2::STATUS_INDEX_RENAMED) => Yellow.paint("R"),
            s if s.contains(git2::STATUS_INDEX_TYPECHANGE) => Purple.paint("T"),
            _ => GREY.paint("-"),
        }
    }
}

#[cfg(not(feature="git"))]
struct Git;

#[cfg(not(feature="git"))]
impl Git {
    fn scan(_: &Path) -> Result<Git, ()> {
        // Don't do anything without Git support
        Err(())
    }

    fn status(&self, _: &Path) -> String {
        // The Err above means that this should never happen
        panic!("Tried to access a Git repo without Git support!");
    }
}
