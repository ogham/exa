//! Getting the Git status of files and directories.

use std::path::{Path, PathBuf};

use git2;

use fs::fields as f;


/// A **Git cache** is assembled based on the user’s input arguments.
///
/// This uses vectors to avoid the overhead of hashing: it’s not worth it when the
/// expected number of Git repositories per exa invocation is 0 or 1...
pub struct GitCache {

    /// A list of discovered Git repositories and their paths.
    repos: Vec<GitRepo>,

    /// Paths that we’ve confirmed do not have Git repositories underneath them.
    misses: Vec<PathBuf>,
}


/// A **Git repository** is one we’ve discovered somewhere on the filesystem.
pub struct GitRepo {

    /// Most of the interesting Git stuff goes through this.
    repo: git2::Repository,

    /// The working directory of this repository.
    /// This is used to check whether two repositories are the same.
    workdir: PathBuf,

    /// The path that was originally checked to discover this repository.
    /// This is as important as the extra_paths (it gets checked first), but
    /// is separate to avoid having to deal with a non-empty Vec.
    original_path: PathBuf,

    /// Any other paths that were checked only to result in this same
    /// repository.
    extra_paths: Vec<PathBuf>,
}

impl GitRepo {
    fn has_workdir(&self, path: &Path) -> bool {
        self.workdir == path
    }

    fn has_path(&self, path: &Path) -> bool {
        self.original_path == path || self.extra_paths.iter().any(|e| e == path)
    }
}

use std::iter::FromIterator;
impl FromIterator<PathBuf> for GitCache {
    fn from_iter<I: IntoIterator<Item=PathBuf>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut git = GitCache {
            repos: Vec::with_capacity(iter.size_hint().0),
            misses: Vec::new(),
        };

        for path in iter {
            if git.misses.contains(&path) {
                debug!("Skipping {:?} because it already came back Gitless", path);
            }
            else if git.repos.iter().any(|e| e.has_path(&path)) {
                debug!("Skipping {:?} because we already queried it", path);
            }
            else {
                match GitRepo::discover(path) {
                    Ok(r) => {
                        if let Some(mut r2) = git.repos.iter_mut().find(|e| e.has_workdir(&r.workdir)) {
                            debug!("Adding to existing repo (workdir matches with {:?})", r2.workdir);
                            r2.extra_paths.push(r.original_path);
                            continue;
                        }

                        debug!("Creating new repo in cache");
                        git.repos.push(r);
                    },
                    Err(miss) => git.misses.push(miss),
                }
            }
        }

        git
    }
}

impl GitRepo {
    fn discover(path: PathBuf) -> Result<GitRepo, PathBuf> {
        info!("Searching for Git repository above {:?}", path);

        let repo = match git2::Repository::discover(&path) {
            Ok(r) => r,
            Err(e) => {
                error!("Error discovering Git repositories: {:?}", e);
                return Err(path);
            }
        };

        match repo.workdir().map(|wd| wd.to_path_buf()) {
            Some(workdir) => Ok(GitRepo { repo, workdir, original_path: path, extra_paths: Vec::new() }),
            None => {
                warn!("Repository has no workdir?");
                Err(path)
            }
        }
    }
}

impl GitCache {

    /// Gets a repository from the cache and scans it to get all its files’ statuses.
    pub fn get(&self, index: &Path) -> Option<Git> {
        let repo = match self.repos.iter().find(|e| e.has_path(index)) {
            Some(r) => r,
            None    => return None,
        };

        info!("Getting Git statuses for repo with workdir {:?}", &repo.workdir);
        let iter = match repo.repo.statuses(None) {
            Ok(es) => es,
            Err(e) => {
                error!("Error looking up Git statuses: {:?}", e);
                return None;
            }
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


/// Container of Git statuses for all the files in this folder’s Git repository.
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
    /// directories, which don’t really have an ‘official’ status.
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

/// The character to display if the file has been modified and the change
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
