use getopts;

use options::misfire::Misfire;
use fs::dir_action::{DirAction, RecurseOptions};



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

            (_   ,  _,     true )  => Ok(DirAction::Recurse(RecurseOptions::deduce(matches, true)?)),
            (true,  false, false)  => Ok(DirAction::Recurse(RecurseOptions::deduce(matches, false)?)),
            (false, true,  _    )  => Ok(DirAction::AsFile),
            (false, false, _    )  => Ok(DirAction::List),
        }
    }
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

        Ok(RecurseOptions { tree, max_depth })
    }
}
