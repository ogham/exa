use options::parser::MatchedFlags;
use options::{flags, Misfire};

use fs::dir_action::{DirAction, RecurseOptions};


impl DirAction {

    /// Determine which action to perform when trying to list a directory.
    pub fn deduce(matches: &MatchedFlags) -> Result<DirAction, Misfire> {
        let recurse = matches.has(&flags::RECURSE)?;
        let list    = matches.has(&flags::LIST_DIRS)?;
        let tree    = matches.has(&flags::TREE)?;

        // Early check for --level when it wouldn’t do anything
        if !recurse && !tree && matches.get(&flags::LEVEL)?.is_some() {
            return Err(Misfire::Useless2(&flags::LEVEL, &flags::RECURSE, &flags::TREE));
        }

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
    pub fn deduce(matches: &MatchedFlags, tree: bool) -> Result<RecurseOptions, Misfire> {
        let max_depth = if let Some(level) = matches.get(&flags::LEVEL)? {
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


#[cfg(test)]
mod test {
    use super::*;
    use options::flags;
    use options::parser::Flag;

    macro_rules! test {
        ($name:ident: $type:ident <- $inputs:expr; $stricts:expr => $result:expr) => {
            #[test]
            fn $name() {
                use options::parser::Arg;
                use options::test::parse_for_test;
                use options::test::Strictnesses::*;

                static TEST_ARGS: &[&Arg] = &[&flags::RECURSE, &flags::LIST_DIRS, &flags::TREE, &flags::LEVEL ];
                for result in parse_for_test($inputs.as_ref(), TEST_ARGS, $stricts, |mf| $type::deduce(mf)) {
                    assert_eq!(result, $result);
                }
            }
        };
    }


    // Default behaviour
    test!(empty:           DirAction <- [];               Both => Ok(DirAction::List));

    // Listing files as directories
    test!(dirs_short:      DirAction <- ["-d"];           Both => Ok(DirAction::AsFile));
    test!(dirs_long:       DirAction <- ["--list-dirs"];  Both => Ok(DirAction::AsFile));

    // Recursing
    use self::DirAction::Recurse;
    test!(rec_short:       DirAction <- ["-R"];                           Both => Ok(Recurse(RecurseOptions { tree: false, max_depth: None })));
    test!(rec_long:        DirAction <- ["--recurse"];                    Both => Ok(Recurse(RecurseOptions { tree: false, max_depth: None })));
    test!(rec_lim_short:   DirAction <- ["-RL4"];                         Both => Ok(Recurse(RecurseOptions { tree: false, max_depth: Some(4) })));
    test!(rec_lim_short_2: DirAction <- ["-RL=5"];                        Both => Ok(Recurse(RecurseOptions { tree: false, max_depth: Some(5) })));
    test!(rec_lim_long:    DirAction <- ["--recurse", "--level", "666"];  Both => Ok(Recurse(RecurseOptions { tree: false, max_depth: Some(666) })));
    test!(rec_lim_long_2:  DirAction <- ["--recurse", "--level=0118"];    Both => Ok(Recurse(RecurseOptions { tree: false, max_depth: Some(118) })));
    test!(rec_tree:        DirAction <- ["--recurse", "--tree"];          Both => Ok(Recurse(RecurseOptions { tree: true,  max_depth: None })));
    test!(rec_short_tree:  DirAction <- ["--tree", "--recurse"];          Both => Ok(Recurse(RecurseOptions { tree: true,  max_depth: None })));

    // Errors
    test!(error:           DirAction <- ["--list-dirs", "--recurse"];     Both => Err(Misfire::Conflict(&flags::RECURSE, &flags::LIST_DIRS)));
    test!(error_2:         DirAction <- ["--list-dirs", "--tree"];        Both => Err(Misfire::Conflict(&flags::TREE,    &flags::LIST_DIRS)));
    test!(underwaterlevel: DirAction <- ["--level=4"];                    Both => Err(Misfire::Useless2(&flags::LEVEL, &flags::RECURSE, &flags::TREE)));

    // Overriding
    test!(overriding_1:    DirAction <- ["-RL=6", "-L=7"];                Last => Ok(Recurse(RecurseOptions { tree: false, max_depth: Some(7) })));
    test!(overriding_2:    DirAction <- ["-RL=6", "-L=7"];            Complain => Err(Misfire::Duplicate(Flag::Short(b'L'), Flag::Short(b'L'))));
}
