//! What to do when encountering a directory?

/// The action to take when trying to list a file that turns out to be a
/// directory.
///
/// By default, exa will display the information about files passed in as
/// command-line arguments, with one file per entry. However, if a directory
/// is passed in, exa assumes that the user wants to see its contents, rather
/// than the directory itself.
///
/// This can get annoying sometimes: if a user does `exa ~/Downloads/img-*`
/// to see the details of every file starting with `img-`, any directories
/// that happen to start with the same will be listed after the files at
/// the end in a separate block. By listing directories as files, their
/// directory status will be ignored, and both will be listed side-by-side.
///
/// These two modes have recursive analogues in the “recurse” and “tree”
/// modes. Here, instead of just listing the directories, exa will descend
/// into them and print out their contents. The recurse mode does this by
/// having extra output blocks at the end, while the tree mode will show
/// directories inline, with their contents immediately underneath.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum DirAction {

    /// This directory should be listed along with the regular files, instead
    /// of having its contents queried.
    AsFile,

    /// This directory should not be listed, and should instead be opened and
    /// *its* files listed separately. This is the default behaviour.
    List,

    /// This directory should be listed along with the regular files, and then
    /// its contents should be listed afterward. The recursive contents of
    /// *those* contents are dictated by the options argument.
    Recurse(RecurseOptions),
}

impl DirAction {

    /// Gets the recurse options, if this dir action has any.
    pub fn recurse_options(self) -> Option<RecurseOptions> {
        match self {
            Self::Recurse(o)  => Some(o),
            _                 => None,
        }
    }

    /// Whether to treat directories as regular files or not.
    pub fn treat_dirs_as_files(self) -> bool {
        match self {
            Self::AsFile      => true,
            Self::Recurse(o)  => o.tree,
            Self::List        => false,
        }
    }
}


/// The options that determine how to recurse into a directory.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct RecurseOptions {

    /// Whether recursion should be done as a tree or as multiple individual
    /// views of files.
    pub tree: bool,

    /// The maximum number of times that recursion should descend to, if one
    /// is specified.
    pub max_depth: Option<usize>,
}

impl RecurseOptions {

    /// Returns whether a directory of the given depth would be too deep.
    pub fn is_too_deep(self, depth: usize) -> bool {
        match self.max_depth {
            None     => false,
            Some(d)  => d <= depth
        }
    }
}
