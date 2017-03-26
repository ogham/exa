use std::path::{Path, PathBuf};

use git2;

use fs::fields as f;


/// Container of Git statuses for all the files in this folder's Git repository.
pub struct Git {
    statuses: Vec<(PathBuf, git2::Status)>,
}

impl Git {

    /// Discover a Git repository on or above this directory, scanning it for
    /// the files' statuses if one is found.
    pub fn scan(path: &Path) -> Result<Git, git2::Error> {
        let repo = git2::Repository::discover(path)?;
        let workdir = match repo.workdir() {
            Some(w) => w,
            None => return Ok(Git { statuses: vec![] }),  // bare repo
        };

        let statuses = repo.statuses(None)?.iter()
                                                .map(|e| (workdir.join(Path::new(e.path().unwrap())), e.status()))
                                                .collect();

        Ok(Git { statuses: statuses })
    }

    /// Get the status for the file at the given path, if present.
    pub fn status(&self, path: &Path) -> f::Git {
        let status = self.statuses.iter()
                                  .find(|p| p.0.as_path() == path);
        match status {
            Some(&(_, s)) => f::Git { staged: index_status(s),           unstaged: working_tree_status(s) },
            None          => f::Git { staged: f::GitStatus::NotModified, unstaged: f::GitStatus::NotModified }
        }
    }

    /// Get the combined status for all the files whose paths begin with the
    /// path that gets passed in. This is used for getting the status of
    /// directories, which don't really have an 'official' status.
    pub fn dir_status(&self, dir: &Path) -> f::Git {
        let s = self.statuses.iter()
                             .filter(|p| p.0.starts_with(dir))
                             .fold(git2::Status::empty(), |a, b| a | b.1);

        f::Git { staged: index_status(s), unstaged: working_tree_status(s) }
    }
}

/// The character to display if the file has been modified, but not staged.
fn working_tree_status(status: git2::Status) -> f::GitStatus {
    match status {
        s if s.contains(git2::STATUS_WT_NEW)         => f::GitStatus::New,
        s if s.contains(git2::STATUS_WT_MODIFIED)    => f::GitStatus::Modified,
        s if s.contains(git2::STATUS_WT_DELETED)     => f::GitStatus::Deleted,
        s if s.contains(git2::STATUS_WT_RENAMED)     => f::GitStatus::Renamed,
        s if s.contains(git2::STATUS_WT_TYPECHANGE)  => f::GitStatus::TypeChange,
        _                                            => f::GitStatus::NotModified,
    }
}

/// The character to display if the file has been modified, and the change
/// has been staged.
fn index_status(status: git2::Status) -> f::GitStatus {
    match status {
        s if s.contains(git2::STATUS_INDEX_NEW)         => f::GitStatus::New,
        s if s.contains(git2::STATUS_INDEX_MODIFIED)    => f::GitStatus::Modified,
        s if s.contains(git2::STATUS_INDEX_DELETED)     => f::GitStatus::Deleted,
        s if s.contains(git2::STATUS_INDEX_RENAMED)     => f::GitStatus::Renamed,
        s if s.contains(git2::STATUS_INDEX_TYPECHANGE)  => f::GitStatus::TypeChange,
        _                                               => f::GitStatus::NotModified,
    }
}
