/// What to do when encountering a directory?
#[derive(PartialEq, Debug, Copy, Clone)]
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
    pub fn recurse_options(&self) -> Option<RecurseOptions> {
        match *self {
            DirAction::Recurse(opts) => Some(opts),
            _ => None,
        }
    }

    /// Whether to treat directories as regular files or not.
    pub fn treat_dirs_as_files(&self) -> bool {
        match *self {
            DirAction::AsFile => true,
            DirAction::Recurse(RecurseOptions { tree, .. }) => tree,
            _ => false,
        }
    }
}


/// The options that determine how to recurse into a directory.
#[derive(PartialEq, Debug, Copy, Clone)]
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
    pub fn is_too_deep(&self, depth: usize) -> bool {
        match self.max_depth {
            None    => false,
            Some(d) => {
                d <= depth
            }
        }
    }
}