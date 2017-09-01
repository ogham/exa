//! Getting the Git status of files and directories.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

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

impl GitCache {
    pub fn has_anything_for(&self, index: &Path) -> bool {
        self.repos.iter().any(|e| e.has_path(index))
    }

    pub fn get(&self, index: &Path, prefix_lookup: bool) -> f::Git {
        self.repos.iter()
            .find(|e| e.has_path(index))
            .map(|repo| repo.search(index, prefix_lookup))
            .unwrap_or_default()
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

                        debug!("Discovered new Git repo");
                        git.repos.push(r);
                    },
                    Err(miss) => git.misses.push(miss),
                }
            }
        }

        git
    }
}




/// A **Git repository** is one we’ve discovered somewhere on the filesystem.
pub struct GitRepo {

    /// The queryable contents of the repository: either a `git2` repo, or the
    /// cached results from when we queried it last time.
    contents: Mutex<GitContents>,

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

/// A repository’s queried state.
enum GitContents {

    /// All the interesting Git stuff goes through this.
    Before { repo: git2::Repository },

    /// Temporary value used in `repo_to_statuses` so we can move the
    /// repository out of the `Before` variant.
    Processing,

    /// The data we’ve extracted from the repository, but only after we’ve
    /// actually done so.
    After { statuses: Git }
}

impl GitRepo {

    /// Searches through this repository for a path (to a file or directory,
    /// depending on the prefix-lookup flag) and returns its Git status.
    ///
    /// Actually querying the `git2` repository for the mapping of paths to
    /// Git statuses is only done once, and gets cached so we don't need to
    /// re-query the entire repository the times after that.
    ///
    /// The temporary `Processing` enum variant is used after the `git2`
    /// repository is moved out, but before the results have been moved in!
    /// See https://stackoverflow.com/q/45985827/3484614
    fn search(&self, index: &Path, prefix_lookup: bool) -> f::Git {
        use self::GitContents::*;
        use std::mem::replace;

        let mut contents = self.contents.lock().unwrap();
        if let After { ref statuses } = *contents {
            debug!("Git repo {:?} has been found in cache", &self.workdir);
            return statuses.status(index, prefix_lookup);
        }

        debug!("Querying Git repo {:?} for the first time", &self.workdir);
        let repo = replace(&mut *contents, Processing).inner_repo();
        let statuses = repo_to_statuses(repo, &self.workdir);
        let result = statuses.status(index, prefix_lookup);
        let _processing = replace(&mut *contents, After { statuses });
        result
    }

    /// Whether this repository has the given working directory.
    fn has_workdir(&self, path: &Path) -> bool {
        self.workdir == path
    }

    /// Whether this repository cares about the given path at all.
    fn has_path(&self, path: &Path) -> bool {
        path.starts_with(&self.original_path) || self.extra_paths.iter().any(|e| path.starts_with(e))
    }

    /// Searches for a Git repository at any point above the given path.
    /// Returns the original buffer if none is found.
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
            Some(workdir) => {
                let contents = Mutex::new(GitContents::Before { repo });
                Ok(GitRepo { contents, workdir, original_path: path, extra_paths: Vec::new() })
            },
            None => {
                warn!("Repository has no workdir?");
                Err(path)
            }
        }
    }
}


impl GitContents {
    /// Assumes that the repository hasn’t been queried, and extracts it
    /// (consuming the value) if it has. This is needed because the entire
    /// enum variant gets replaced when a repo is queried (see above).
    fn inner_repo(self) -> git2::Repository {
        if let GitContents::Before { repo } = self {
            repo
        }
        else {
            unreachable!("Tried to extract a non-Repository")
        }
    }
}

/// Iterates through a repository’s statuses, consuming it and returning the
/// mapping of files to their Git status.
/// We will have already used the working directory at this point, so it gets
/// passed in rather than deriving it from the `Repository` again.
fn repo_to_statuses(repo: git2::Repository, workdir: &Path) -> Git {
    let mut statuses = Vec::new();

    info!("Getting Git statuses for repo with workdir {:?}", workdir);
    match repo.statuses(None) {
        Ok(es) => {
            for e in es.iter() {
                let path = workdir.join(Path::new(e.path().unwrap()));
                let elem = (path, e.status());
                statuses.push(elem);
            }
        },
        Err(e) => error!("Error looking up Git statuses: {:?}", e),
    }

    Git { statuses }
}


/// Container of Git statuses for all the files in this folder’s Git repository.
struct Git {
    statuses: Vec<(PathBuf, git2::Status)>,
}

impl Git {

    /// Get either the file or directory status for the given path.
    /// “Prefix lookup” means that it should report an aggregate status of all
    /// paths starting with the given prefix (in other words, a directory).
    fn status(&self, index: &Path, prefix_lookup: bool) -> f::Git {
        if prefix_lookup { self.dir_status(index) }
                    else { self.file_status(index) }
    }

    /// Get the status for the file at the given path.
    fn file_status(&self, file: &Path) -> f::Git {
        let path = reorient(file);
        self.statuses.iter()
            .find(|p| p.0.as_path() == path)
            .map(|&(_, s)| f::Git { staged: index_status(s), unstaged: working_tree_status(s) })
            .unwrap_or_default()
    }

    /// Get the combined status for all the files whose paths begin with the
    /// path that gets passed in. This is used for getting the status of
    /// directories, which don’t really have an ‘official’ status.
    fn dir_status(&self, dir: &Path) -> f::Git {
        let path = reorient(dir);
        let s = self.statuses.iter()
                             .filter(|p| p.0.starts_with(&path))
                             .fold(git2::Status::empty(), |a, b| a | b.1);

        f::Git { staged: index_status(s), unstaged: working_tree_status(s) }
    }
}

/// Converts a path to an absolute path based on the current directory.
/// Paths need to be absolute for them to be compared properly, otherwise
/// you’d ask a repo about “./README.md” but it only knows about
/// “/vagrant/REAMDE.md”, prefixed by the workdir.
fn reorient(path: &Path) -> PathBuf {
    use std::env::current_dir;
    // I’m not 100% on this func tbh
    match current_dir() {
        Err(_)  => Path::new(".").join(&path),
        Ok(dir) => dir.join(&path),
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
