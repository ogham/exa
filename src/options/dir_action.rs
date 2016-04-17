use getopts;

use options::misfire::Misfire;


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

    /// Determine which action to perform when trying to list a directory.
    pub fn deduce(matches: &getopts::Matches) -> Result<DirAction, Misfire> {
        let recurse = matches.opt_present("recurse");
        let list    = matches.opt_present("list-dirs");
        let tree    = matches.opt_present("tree");

        match (recurse, list, tree) {

            // You can't --list-dirs along with --recurse or --tree because
            // they already automatically list directories.
            (true,  true,  _    )  => Err(Misfire::Conflict("recurse", "list-dirs")),
            (_,     true,  true )  => Err(Misfire::Conflict("tree", "list-dirs")),

            (_   ,  _,     true )  => Ok(DirAction::Recurse(try!(RecurseOptions::deduce(matches, true)))),
            (true,  false, false)  => Ok(DirAction::Recurse(try!(RecurseOptions::deduce(matches, false)))),
            (false, true,  _    )  => Ok(DirAction::AsFile),
            (false, false, _    )  => Ok(DirAction::List),
        }
    }

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

    /// Determine which files should be recursed into.
    pub fn deduce(matches: &getopts::Matches, tree: bool) -> Result<RecurseOptions, Misfire> {
        let max_depth = if let Some(level) = matches.opt_str("level") {
            match level.parse() {
                Ok(l)   => Some(l),
                Err(e)  => return Err(Misfire::FailedParse(e)),
            }
        }
        else {
            None
        };

        Ok(RecurseOptions {
            tree: tree,
            max_depth: max_depth,
        })
    }

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