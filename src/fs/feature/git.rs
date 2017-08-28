use std::collections::HashMap;
use std::path::{Path, PathBuf};

use git2;

use fs::fields as f;


pub struct GitCache {
    repos: HashMap<PathBuf, Option<GitRepo>>,
}

pub struct GitRepo {
    repo: git2::Repository,
    workdir: PathBuf,
}

impl GitRepo {
    fn discover(path: &Path) -> Option<GitRepo> {
    	info!("Searching for Git repository above {:?}", path);
        if let Ok(repo) = git2::Repository::discover(&path) {
            if let Some(workdir) = repo.workdir().map(|wd| wd.to_path_buf()) {
                return Some(GitRepo { repo, workdir });
            }
        }

        None
    }
}

use std::iter::FromIterator;
impl FromIterator<PathBuf> for GitCache {
    fn from_iter<I: IntoIterator<Item=PathBuf>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut repos = HashMap::with_capacity(iter.size_hint().0);

        for path in iter {
            if repos.contains_key(&path) {
            	debug!("Skipping {:?} because we already queried it", path);
            }
            else {
                let repo = GitRepo::discover(&path);
                let _ = repos.insert(path, repo);
            }
        }

        GitCache { repos }
    }
}

impl GitCache {
    pub fn get(&self, index: &Path) -> Option<Git> {
        let repo = match self.repos[index] {
            Some(ref r) => r,
            None => return None,
        };

        let iter = match repo.repo.statuses(None) {
            Ok(es) => es,
            Err(_) => return None,
        };

        let mut statuses = Vec::new();

        for e in iter.iter() {
            let path = repo.workdir.join(Path::new(e.path().unwrap()));
            let elem = (path, e.status());
            statuses.push(elem);
        }

        Some(Git { statuses })
    }
}


/// Container of Git statuses for all the files in this folder's Git repository.
pub struct Git {
    statuses: Vec<(PathBuf, git2::Status)>,
}

impl Git {

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
