use options::parser::Matches;
use options::{flags, Misfire};

use fs::dir_action::{DirAction, RecurseOptions};


impl DirAction {

    /// Determine which action to perform when trying to list a directory.
    pub fn deduce(matches: &Matches) -> Result<DirAction, Misfire> {
        let recurse = matches.has(&flags::RECURSE);
        let list    = matches.has(&flags::LIST_DIRS);
        let tree    = matches.has(&flags::TREE);

        match (recurse, list, tree) {

            // You can't --list-dirs along with --recurse or --tree because
            // they already automatically list directories.
            (true,  true,  _    )  => Err(Misfire::Conflict(&flags::RECURSE, &flags::LIST_DIRS)),
            (_,     true,  true )  => Err(Misfire::Conflict(&flags::TREE,    &flags::LIST_DIRS)),

            (_   ,  _,     true )  => Ok(DirAction::Recurse(RecurseOptions::deduce(matches, true)?)),
            (true,  false, false)  => Ok(DirAction::Recurse(RecurseOptions::deduce(matches, false)?)),
            (false, true,  _    )  => Ok(DirAction::AsFile),
            (false, false, _    )  => Ok(DirAction::List),
        }
    }
}


impl RecurseOptions {

    /// Determine which files should be recursed into.
    pub fn deduce(matches: &Matches, tree: bool) -> Result<RecurseOptions, Misfire> {
        let max_depth = if let Some(level) = matches.get(&flags::LEVEL) {
            match level.to_string_lossy().parse() {
                Ok(l)   => Some(l),
                Err(e)  => return Err(Misfire::FailedParse(e)),
            }
        }
        else {
            None
        };

        Ok(RecurseOptions { tree, max_depth })
    }
}
