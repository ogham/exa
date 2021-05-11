//! Getting the Git status of files and directories.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use log::*;

use crate::fs::fields as f;


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
    fn from_iter<I>(iter: I) -> Self
    where I: IntoIterator<Item=PathBuf>
    {
        let iter = iter.into_iter();
        let mut git = Self {
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
                        if let Some(r2) = git.repos.iter_mut().find(|e| e.has_workdir(&r.workdir)) {
                            debug!("Adding to existing repo (workdir matches with {:?})", r2.workdir);
                            r2.extra_paths.push(r.original_path);
                            continue;
                        }

                        debug!("Discovered new Git repo");
                        git.repos.push(r);
                    }
                    Err(miss) => {
                        git.misses.push(miss)
                    }
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
    Before {
        repo: git2::Repository,
    },

    /// Temporary value used in `repo_to_statuses` so we can move the
    /// repository out of the `Before` variant.
    Processing,

    /// The data we’ve extracted from the repository, but only after we’ve
    /// actually done so.
    After {
        statuses: Git,
    },
}

impl GitRepo {

    /// Searches through this repository for a path (to a file or directory,
    /// depending on the prefix-lookup flag) and returns its Git status.
    ///
    /// Actually querying the `git2` repository for the mapping of paths to
    /// Git statuses is only done once, and gets cached so we don’t need to
    /// re-query the entire repository the times after that.
    ///
    /// The temporary `Processing` enum variant is used after the `git2`
    /// repository is moved out, but before the results have been moved in!
    /// See <https://stackoverflow.com/q/45985827/3484614>
    fn search(&self, index: &Path, prefix_lookup: bool) -> f::Git {
        use std::mem::replace;

        let mut contents = self.contents.lock().unwrap();
        if let GitContents::After { ref statuses } = *contents {
            debug!("Git repo {:?} has been found in cache", &self.workdir);
            return statuses.status(index, prefix_lookup);
        }

        debug!("Querying Git repo {:?} for the first time", &self.workdir);
        let repo = replace(&mut *contents, GitContents::Processing).inner_repo();
        let statuses = repo_to_statuses(&repo, &self.workdir);
        let result = statuses.status(index, prefix_lookup);
        let _processing = replace(&mut *contents, GitContents::After { statuses });
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
    fn discover(path: PathBuf) -> Result<Self, PathBuf> {
        info!("Searching for Git repository above {:?}", path);
        let repo = match git2::Repository::discover(&path) {
            Ok(r) => r,
            Err(e) => {
                error!("Error discovering Git repositories: {:?}", e);
                return Err(path);
            }
        };

        if let Some(workdir) = repo.workdir() {
            let workdir = workdir.to_path_buf();
            let contents = Mutex::new(GitContents::Before { repo });
            Ok(Self { contents, workdir, original_path: path, extra_paths: Vec::new() })
        }
        else {
            warn!("Repository has no workdir?");
            Err(path)
        }
    }
}


impl GitContents {
    /// Assumes that the repository hasn’t been queried, and extracts it
    /// (consuming the value) if it has. This is needed because the entire
    /// enum variant gets replaced when a repo is queried (see above).
    fn inner_repo(self) -> git2::Repository {
        if let Self::Before { repo } = self {
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
fn repo_to_statuses(repo: &git2::Repository, workdir: &Path) -> Git {
    let mut statuses = Vec::new();

    info!("Getting Git statuses for repo with workdir {:?}", workdir);
    match repo.statuses(None) {
        Ok(es) => {
            for e in es.iter() {
                let path = workdir.join(Path::new(e.path().unwrap()));
                let elem = (path, e.status());
                statuses.push(elem);
            }
        }
        Err(e) => {
            error!("Error looking up Git statuses: {:?}", e);
        }
    }

    Git { statuses }
}

// The `repo.statuses` call above takes a long time. exa debug output:
//
//   20.311276  INFO:exa::fs::feature::git: Getting Git statuses for repo with workdir "/vagrant/"
//   20.799610  DEBUG:exa::output::table: Getting Git status for file "./Cargo.toml"
//
// Even inserting another logging line immediately afterwards doesn’t make it
// look any faster.


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

    /// Get the user-facing status of a file.
    /// We check the statuses directly applying to a file, and for the ignored
    /// status we check if any of its parents directories is ignored by git.
    fn file_status(&self, file: &Path) -> f::Git {
        let path = reorient(file);

        let s = self.statuses.iter()
            .filter(|p| if p.1 == git2::Status::IGNORED {
                path.starts_with(&p.0)
            } else {
                p.0 == path
            })
            .fold(git2::Status::empty(), |a, b| a | b.1);

        let staged = index_status(s);
        let unstaged = working_tree_status(s);
        f::Git { staged, unstaged }
    }

    /// Get the combined, user-facing status of a directory.
    /// Statuses are aggregating (for example, a directory is considered
    /// modified if any file under it has the status modified), except for
    /// ignored status which applies to files under (for example, a directory
    /// is considered ignored if one of its parent directories is ignored).
    fn dir_status(&self, dir: &Path) -> f::Git {
        let path = reorient(dir);

        let s = self.statuses.iter()
            .filter(|p| if p.1 == git2::Status::IGNORED {
                path.starts_with(&p.0)
            } else {
                p.0.starts_with(&path)
            })
            .fold(git2::Status::empty(), |a, b| a | b.1);

        let staged = index_status(s);
        let unstaged = working_tree_status(s);
        f::Git { staged, unstaged }
    }
}


/// Converts a path to an absolute path based on the current directory.
/// Paths need to be absolute for them to be compared properly, otherwise
/// you’d ask a repo about “./README.md” but it only knows about
/// “/vagrant/README.md”, prefixed by the workdir.
fn reorient(path: &Path) -> PathBuf {
    let path = std::env::current_dir()
        .unwrap()
        .with_file_name(&path);

    path.canonicalize().unwrap_or(path)
}

/// The character to display if the file has been modified, but not staged.
fn working_tree_status(status: git2::Status) -> f::GitStatus {
    match status {
        s if s.contains(git2::Status::WT_NEW)         => f::GitStatus::New,
        s if s.contains(git2::Status::WT_MODIFIED)    => f::GitStatus::Modified,
        s if s.contains(git2::Status::WT_DELETED)     => f::GitStatus::Deleted,
        s if s.contains(git2::Status::WT_RENAMED)     => f::GitStatus::Renamed,
        s if s.contains(git2::Status::WT_TYPECHANGE)  => f::GitStatus::TypeChange,
        s if s.contains(git2::Status::IGNORED)        => f::GitStatus::Ignored,
        s if s.contains(git2::Status::CONFLICTED)     => f::GitStatus::Conflicted,
        _                                             => f::GitStatus::NotModified,
    }
}

/// The character to display if the file has been modified and the change
/// has been staged.
fn index_status(status: git2::Status) -> f::GitStatus {
    match status {
        s if s.contains(git2::Status::INDEX_NEW)         => f::GitStatus::New,
        s if s.contains(git2::Status::INDEX_MODIFIED)    => f::GitStatus::Modified,
        s if s.contains(git2::Status::INDEX_DELETED)     => f::GitStatus::Deleted,
        s if s.contains(git2::Status::INDEX_RENAMED)     => f::GitStatus::Renamed,
        s if s.contains(git2::Status::INDEX_TYPECHANGE)  => f::GitStatus::TypeChange,
        _                                                => f::GitStatus::NotModified,
    }
}
