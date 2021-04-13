use crate::fs::feature::xattr;
use crate::options::{flags, OptionsError, NumberSource, Vars};
use crate::options::parser::MatchedFlags;
use crate::output::{View, Mode, TerminalWidth, grid, details};
use crate::output::grid_details::{self, RowThreshold};
use crate::output::file_name::Options as FileStyle;
use crate::output::table::{TimeTypes, SizeFormat, UserFormat, Columns, Options as TableOptions};
use crate::output::time::TimeFormat;


impl View {
    pub fn deduce<V: Vars>(matches: &MatchedFlags<'_>, vars: &V) -> Result<Self, OptionsError> {
        let mode = Mode::deduce(matches, vars)?;
        let width = TerminalWidth::deduce(vars)?;
        let file_style = FileStyle::deduce(matches, vars)?;
        Ok(Self { mode, width, file_style })
    }
}


impl Mode {

    /// Determine which viewing mode to use based on the user’s options.
    ///
    /// As with the other options, arguments are scanned right-to-left and the
    /// first flag found is matched, so `exa --oneline --long` will pick a
    /// details view, and `exa --long --oneline` will pick the lines view.
    ///
    /// This is complicated a little by the fact that `--grid` and `--tree`
    /// can also combine with `--long`, so care has to be taken to use the
    pub fn deduce<V: Vars>(matches: &MatchedFlags<'_>, vars: &V) -> Result<Self, OptionsError> {
        let flag = matches.has_where_any(|f| f.matches(&flags::LONG) || f.matches(&flags::ONE_LINE)
                                          || f.matches(&flags::GRID) || f.matches(&flags::TREE));

        let flag = match flag {
            Some(f) => f,
            None => {
                Self::strict_check_long_flags(matches)?;
                let grid = grid::Options::deduce(matches)?;
                return Ok(Self::Grid(grid));
            }
        };

        if flag.matches(&flags::LONG)
        || (flag.matches(&flags::TREE) && matches.has(&flags::LONG)?)
        || (flag.matches(&flags::GRID) && matches.has(&flags::LONG)?)
        {
            let _ = matches.has(&flags::LONG)?;
            let details = details::Options::deduce_long(matches, vars)?;

            let flag = matches.has_where_any(|f| f.matches(&flags::GRID) || f.matches(&flags::TREE));

            if flag.is_some() && flag.unwrap().matches(&flags::GRID) {
                let _ = matches.has(&flags::GRID)?;
                let grid = grid::Options::deduce(matches)?;
                let row_threshold = RowThreshold::deduce(vars)?;
                let grid_details = grid_details::Options { grid, details, row_threshold };
                return Ok(Self::GridDetails(grid_details));
            }
            else {
                // the --tree case is handled by the DirAction parser later
                return Ok(Self::Details(details));
            }
        }

        Self::strict_check_long_flags(matches)?;

        if flag.matches(&flags::TREE) {
            let _ = matches.has(&flags::TREE)?;
            let details = details::Options::deduce_tree(matches)?;
            return Ok(Self::Details(details));
        }

        if flag.matches(&flags::ONE_LINE) {
            let _ = matches.has(&flags::ONE_LINE)?;
            return Ok(Self::Lines);
        }

        let grid = grid::Options::deduce(matches)?;
        Ok(Self::Grid(grid))
    }

    fn strict_check_long_flags(matches: &MatchedFlags<'_>) -> Result<(), OptionsError> {
        // If --long hasn’t been passed, then check if we need to warn the
        // user about flags that won’t have any effect.
        if matches.is_strict() {
            for option in &[ &flags::BINARY, &flags::BYTES, &flags::INODE, &flags::LINKS,
                             &flags::HEADER, &flags::BLOCKS, &flags::TIME, &flags::GROUP, &flags::NUMERIC ] {
                if matches.has(option)? {
                    return Err(OptionsError::Useless(*option, false, &flags::LONG));
                }
            }

            if matches.has(&flags::GIT)? {
                return Err(OptionsError::Useless(&flags::GIT, false, &flags::LONG));
            }
            else if matches.has(&flags::LEVEL)? && ! matches.has(&flags::RECURSE)? && ! matches.has(&flags::TREE)? {
                return Err(OptionsError::Useless2(&flags::LEVEL, &flags::RECURSE, &flags::TREE));
            }
        }

        Ok(())
    }
}


impl grid::Options {
    fn deduce(matches: &MatchedFlags<'_>) -> Result<Self, OptionsError> {
        let grid = grid::Options {
            across: matches.has(&flags::ACROSS)?,
        };

        Ok(grid)
    }
}


impl details::Options {
    fn deduce_tree(matches: &MatchedFlags<'_>) -> Result<Self, OptionsError> {
        let details = details::Options {
            table: None,
            header: false,
            xattr: xattr::ENABLED && matches.has(&flags::EXTENDED)?,
        };

        Ok(details)
    }

    fn deduce_long<V: Vars>(matches: &MatchedFlags<'_>, vars: &V) -> Result<Self, OptionsError> {
        if matches.is_strict() {
            if matches.has(&flags::ACROSS)? && ! matches.has(&flags::GRID)? {
                return Err(OptionsError::Useless(&flags::ACROSS, true, &flags::LONG));
            }
            else if matches.has(&flags::ONE_LINE)? {
                return Err(OptionsError::Useless(&flags::ONE_LINE, true, &flags::LONG));
            }
        }

        Ok(details::Options {
            table: Some(TableOptions::deduce(matches, vars)?),
            header: matches.has(&flags::HEADER)?,
            xattr: xattr::ENABLED && matches.has(&flags::EXTENDED)?,
        })
    }
}


impl TerminalWidth {
    fn deduce<V: Vars>(vars: &V) -> Result<Self, OptionsError> {
        use crate::options::vars;

        if let Some(columns) = vars.get(vars::COLUMNS).and_then(|s| s.into_string().ok()) {
            match columns.parse() {
                Ok(width) => {
                    Ok(Self::Set(width))
                }
                Err(e) => {
                    let source = NumberSource::Env(vars::COLUMNS);
                    Err(OptionsError::FailedParse(columns, source, e))
                }
            }
        }
        else {
            Ok(Self::Automatic)
        }
    }
}


impl RowThreshold {
    fn deduce<V: Vars>(vars: &V) -> Result<Self, OptionsError> {
        use crate::options::vars;

        if let Some(columns) = vars.get(vars::EXA_GRID_ROWS).and_then(|s| s.into_string().ok()) {
            match columns.parse() {
                Ok(rows) => {
                    Ok(Self::MinimumRows(rows))
                }
                Err(e) => {
                    let source = NumberSource::Env(vars::EXA_GRID_ROWS);
                    Err(OptionsError::FailedParse(columns, source, e))
                }
            }
        }
        else {
            Ok(Self::AlwaysGrid)
        }
    }
}


impl TableOptions {
    fn deduce<V: Vars>(matches: &MatchedFlags<'_>, vars: &V) -> Result<Self, OptionsError> {
        let time_format = TimeFormat::deduce(matches, vars)?;
        let size_format = SizeFormat::deduce(matches)?;
        let user_format = UserFormat::deduce(matches)?;
        let columns = Columns::deduce(matches)?;
        Ok(Self { time_format, size_format, columns , user_format})
    }
}


impl Columns {
    fn deduce(matches: &MatchedFlags<'_>) -> Result<Self, OptionsError> {
        let time_types = TimeTypes::deduce(matches)?;
        let git = matches.has(&flags::GIT)?;

        let blocks = matches.has(&flags::BLOCKS)?;
        let group  = matches.has(&flags::GROUP)?;
        let inode  = matches.has(&flags::INODE)?;
        let links  = matches.has(&flags::LINKS)?;
        let octal  = matches.has(&flags::OCTAL)?;

        let permissions = ! matches.has(&flags::NO_PERMISSIONS)?;
        let filesize =    ! matches.has(&flags::NO_FILESIZE)?;
        let user =        ! matches.has(&flags::NO_USER)?;

        Ok(Self { time_types, git, octal, blocks, group, inode, links, permissions, filesize, user })
    }
}


impl SizeFormat {

    /// Determine which file size to use in the file size column based on
    /// the user’s options.
    ///
    /// The default mode is to use the decimal prefixes, as they are the
    /// most commonly-understood, and don’t involve trying to parse large
    /// strings of digits in your head. Changing the format to anything else
    /// involves the `--binary` or `--bytes` flags, and these conflict with
    /// each other.
    fn deduce(matches: &MatchedFlags<'_>) -> Result<Self, OptionsError> {
        let flag = matches.has_where(|f| f.matches(&flags::BINARY) || f.matches(&flags::BYTES))?;

        Ok(match flag {
            Some(f) if f.matches(&flags::BINARY)  => Self::BinaryBytes,
            Some(f) if f.matches(&flags::BYTES)   => Self::JustBytes,
            _                                     => Self::DecimalBytes,
        })
    }
}


impl TimeFormat {

    /// Determine how time should be formatted in timestamp columns.
    fn deduce<V: Vars>(matches: &MatchedFlags<'_>, vars: &V) -> Result<Self, OptionsError> {
        let word =
            if let Some(w) = matches.get(&flags::TIME_STYLE)? {
                w.to_os_string()
            }
            else {
                use crate::options::vars;
                match vars.get(vars::TIME_STYLE) {
                    Some(ref t) if ! t.is_empty()  => t.clone(),
                    _                              => return Ok(Self::DefaultFormat)
                }
            };

        if &word == "default" {
            Ok(Self::DefaultFormat)
        }
        else if &word == "iso" {
            Ok(Self::ISOFormat)
        }
        else if &word == "long-iso" {
            Ok(Self::LongISO)
        }
        else if &word == "full-iso" {
            Ok(Self::FullISO)
        }
        else {
            Err(OptionsError::BadArgument(&flags::TIME_STYLE, word))
        }
    }
}


impl UserFormat {
    fn deduce(matches: &MatchedFlags<'_>) -> Result<Self, OptionsError> {
        let flag = matches.has(&flags::NUMERIC)?;
        Ok(if flag { Self::Numeric } else { Self::Name })
    }
}


impl TimeTypes {

    /// Determine which of a file’s time fields should be displayed for it
    /// based on the user’s options.
    ///
    /// There are two separate ways to pick which fields to show: with a
    /// flag (such as `--modified`) or with a parameter (such as
    /// `--time=modified`). An error is signaled if both ways are used.
    ///
    /// It’s valid to show more than one column by passing in more than one
    /// option, but passing *no* options means that the user just wants to
    /// see the default set.
    fn deduce(matches: &MatchedFlags<'_>) -> Result<Self, OptionsError> {
        let possible_word = matches.get(&flags::TIME)?;
        let modified = matches.has(&flags::MODIFIED)?;
        let changed  = matches.has(&flags::CHANGED)?;
        let accessed = matches.has(&flags::ACCESSED)?;
        let created  = matches.has(&flags::CREATED)?;

        let no_time = matches.has(&flags::NO_TIME)?;

        let time_types = if no_time {
            Self { modified: false, changed: false, accessed: false, created: false }
        } else if let Some(word) = possible_word {
            if modified {
                return Err(OptionsError::Useless(&flags::MODIFIED, true, &flags::TIME));
            }
            else if changed {
                return Err(OptionsError::Useless(&flags::CHANGED, true, &flags::TIME));
            }
            else if accessed {
                return Err(OptionsError::Useless(&flags::ACCESSED, true, &flags::TIME));
            }
            else if created {
                return Err(OptionsError::Useless(&flags::CREATED, true, &flags::TIME));
            }
            else if word == "mod" || word == "modified" {
                Self { modified: true,  changed: false, accessed: false, created: false }
            }
            else if word == "ch" || word == "changed" {
                Self { modified: false, changed: true,  accessed: false, created: false }
            }
            else if word == "acc" || word == "accessed" {
                Self { modified: false, changed: false, accessed: true,  created: false }
            }
            else if word == "cr" || word == "created" {
                Self { modified: false, changed: false, accessed: false, created: true  }
            }
            else {
                return Err(OptionsError::BadArgument(&flags::TIME, word.into()));
            }
        }
        else if modified || changed || accessed || created {
            Self { modified, changed, accessed, created }
        }
        else {
            Self::default()
        };

        Ok(time_types)
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::ffi::OsString;
    use crate::options::flags;
    use crate::options::parser::{Flag, Arg};

    use crate::options::test::parse_for_test;
    use crate::options::test::Strictnesses::*;

    static TEST_ARGS: &[&Arg] = &[ &flags::BINARY, &flags::BYTES,    &flags::TIME_STYLE,
                                   &flags::TIME,   &flags::MODIFIED, &flags::CHANGED,
                                   &flags::CREATED, &flags::ACCESSED,
                                   &flags::HEADER, &flags::GROUP,  &flags::INODE, &flags::GIT,
                                   &flags::LINKS,  &flags::BLOCKS, &flags::LONG,  &flags::LEVEL,
                                   &flags::GRID,   &flags::ACROSS, &flags::ONE_LINE, &flags::TREE,
                                   &flags::NUMERIC ];

    macro_rules! test {

        ($name:ident: $type:ident <- $inputs:expr; $stricts:expr => $result:expr) => {
            /// Macro that writes a test.
            /// If testing both strictnesses, they’ll both be done in the same function.
            #[test]
            fn $name() {
                for result in parse_for_test($inputs.as_ref(), TEST_ARGS, $stricts, |mf| $type::deduce(mf)) {
                    assert_eq!(result, $result);
                }
            }
        };

        ($name:ident: $type:ident <- $inputs:expr; $stricts:expr => err $result:expr) => {
            /// Special macro for testing Err results.
            /// This is needed because sometimes the Ok type doesn’t implement PartialEq.
            #[test]
            fn $name() {
                for result in parse_for_test($inputs.as_ref(), TEST_ARGS, $stricts, |mf| $type::deduce(mf)) {
                    assert_eq!(result.unwrap_err(), $result);
                }
            }
        };

        ($name:ident: $type:ident <- $inputs:expr; $stricts:expr => like $pat:pat) => {
            /// More general macro for testing against a pattern.
            /// Instead of using PartialEq, this just tests if it matches a pat.
            #[test]
            fn $name() {
                for result in parse_for_test($inputs.as_ref(), TEST_ARGS, $stricts, |mf| $type::deduce(mf)) {
                    println!("Testing {:?}", result);
                    match result {
                        $pat => assert!(true),
                        _    => assert!(false),
                    }
                }
            }
        };


        ($name:ident: $type:ident <- $inputs:expr, $vars:expr; $stricts:expr => err $result:expr) => {
            /// Like above, but with $vars.
            #[test]
            fn $name() {
                for result in parse_for_test($inputs.as_ref(), TEST_ARGS, $stricts, |mf| $type::deduce(mf, &$vars)) {
                    assert_eq!(result.unwrap_err(), $result);
                }
            }
        };

        ($name:ident: $type:ident <- $inputs:expr, $vars:expr; $stricts:expr => like $pat:pat) => {
            /// Like further above, but with $vars.
            #[test]
            fn $name() {
                for result in parse_for_test($inputs.as_ref(), TEST_ARGS, $stricts, |mf| $type::deduce(mf, &$vars)) {
                    println!("Testing {:?}", result);
                    match result {
                        $pat => assert!(true),
                        _    => assert!(false),
                    }
                }
            }
        };
    }


    mod size_formats {
        use super::*;

        // Default behaviour
        test!(empty:   SizeFormat <- [];                       Both => Ok(SizeFormat::DecimalBytes));

        // Individual flags
        test!(binary:  SizeFormat <- ["--binary"];             Both => Ok(SizeFormat::BinaryBytes));
        test!(bytes:   SizeFormat <- ["--bytes"];              Both => Ok(SizeFormat::JustBytes));

        // Overriding
        test!(both_1:  SizeFormat <- ["--binary", "--binary"];  Last => Ok(SizeFormat::BinaryBytes));
        test!(both_2:  SizeFormat <- ["--bytes",  "--binary"];  Last => Ok(SizeFormat::BinaryBytes));
        test!(both_3:  SizeFormat <- ["--binary", "--bytes"];   Last => Ok(SizeFormat::JustBytes));
        test!(both_4:  SizeFormat <- ["--bytes",  "--bytes"];   Last => Ok(SizeFormat::JustBytes));

        test!(both_5:  SizeFormat <- ["--binary", "--binary"];  Complain => err OptionsError::Duplicate(Flag::Long("binary"), Flag::Long("binary")));
        test!(both_6:  SizeFormat <- ["--bytes",  "--binary"];  Complain => err OptionsError::Duplicate(Flag::Long("bytes"),  Flag::Long("binary")));
        test!(both_7:  SizeFormat <- ["--binary", "--bytes"];   Complain => err OptionsError::Duplicate(Flag::Long("binary"), Flag::Long("bytes")));
        test!(both_8:  SizeFormat <- ["--bytes",  "--bytes"];   Complain => err OptionsError::Duplicate(Flag::Long("bytes"),  Flag::Long("bytes")));
    }


    mod time_formats {
        use super::*;

        // These tests use pattern matching because TimeFormat doesn’t
        // implement PartialEq.

        // Default behaviour
        test!(empty:     TimeFormat <- [], None;                            Both => like Ok(TimeFormat::DefaultFormat));

        // Individual settings
        test!(default:   TimeFormat <- ["--time-style=default"], None;      Both => like Ok(TimeFormat::DefaultFormat));
        test!(iso:       TimeFormat <- ["--time-style", "iso"], None;       Both => like Ok(TimeFormat::ISOFormat));
        test!(long_iso:  TimeFormat <- ["--time-style=long-iso"], None;     Both => like Ok(TimeFormat::LongISO));
        test!(full_iso:  TimeFormat <- ["--time-style", "full-iso"], None;  Both => like Ok(TimeFormat::FullISO));

        // Overriding
        test!(actually:  TimeFormat <- ["--time-style=default", "--time-style", "iso"], None;  Last => like Ok(TimeFormat::ISOFormat));
        test!(actual_2:  TimeFormat <- ["--time-style=default", "--time-style", "iso"], None;  Complain => err OptionsError::Duplicate(Flag::Long("time-style"), Flag::Long("time-style")));

        test!(nevermind: TimeFormat <- ["--time-style", "long-iso", "--time-style=full-iso"], None;  Last => like Ok(TimeFormat::FullISO));
        test!(nevermore: TimeFormat <- ["--time-style", "long-iso", "--time-style=full-iso"], None;  Complain => err OptionsError::Duplicate(Flag::Long("time-style"), Flag::Long("time-style")));

        // Errors
        test!(daily:     TimeFormat <- ["--time-style=24-hour"], None;  Both => err OptionsError::BadArgument(&flags::TIME_STYLE, OsString::from("24-hour")));

        // `TIME_STYLE` environment variable is defined.
        // If the time-style argument is not given, `TIME_STYLE` is used.
        test!(use_env:     TimeFormat <- [], Some("long-iso".into());  Both => like Ok(TimeFormat::LongISO));

        // If the time-style argument is given, `TIME_STYLE` is overriding.
        test!(override_env:     TimeFormat <- ["--time-style=full-iso"], Some("long-iso".into());  Both => like Ok(TimeFormat::FullISO));
    }


    mod time_types {
        use super::*;

        // Default behaviour
        test!(empty:     TimeTypes <- [];                      Both => Ok(TimeTypes::default()));

        // Modified
        test!(modified:  TimeTypes <- ["--modified"];          Both => Ok(TimeTypes { modified: true,  changed: false, accessed: false, created: false }));
        test!(m:         TimeTypes <- ["-m"];                  Both => Ok(TimeTypes { modified: true,  changed: false, accessed: false, created: false }));
        test!(time_mod:  TimeTypes <- ["--time=modified"];     Both => Ok(TimeTypes { modified: true,  changed: false, accessed: false, created: false }));
        test!(t_m:       TimeTypes <- ["-tmod"];               Both => Ok(TimeTypes { modified: true,  changed: false, accessed: false, created: false }));

        // Changed
        #[cfg(target_family = "unix")]
        test!(changed:   TimeTypes <- ["--changed"];           Both => Ok(TimeTypes { modified: false, changed: true,  accessed: false, created: false }));
        #[cfg(target_family = "unix")]
        test!(time_ch:   TimeTypes <- ["--time=changed"];      Both => Ok(TimeTypes { modified: false, changed: true,  accessed: false, created: false }));
        #[cfg(target_family = "unix")]
        test!(t_ch:    TimeTypes <- ["-t", "ch"];              Both => Ok(TimeTypes { modified: false, changed: true,  accessed: false, created: false }));

        // Accessed
        test!(acc:       TimeTypes <- ["--accessed"];          Both => Ok(TimeTypes { modified: false, changed: false, accessed: true,  created: false }));
        test!(a:         TimeTypes <- ["-u"];                  Both => Ok(TimeTypes { modified: false, changed: false, accessed: true,  created: false }));
        test!(time_acc:  TimeTypes <- ["--time", "accessed"];  Both => Ok(TimeTypes { modified: false, changed: false, accessed: true,  created: false }));
        test!(time_a:    TimeTypes <- ["-t", "acc"];           Both => Ok(TimeTypes { modified: false, changed: false, accessed: true,  created: false }));

        // Created
        test!(cr:        TimeTypes <- ["--created"];           Both => Ok(TimeTypes { modified: false, changed: false, accessed: false, created: true  }));
        test!(c:         TimeTypes <- ["-U"];                  Both => Ok(TimeTypes { modified: false, changed: false, accessed: false, created: true  }));
        test!(time_cr:   TimeTypes <- ["--time=created"];      Both => Ok(TimeTypes { modified: false, changed: false, accessed: false, created: true  }));
        test!(t_cr:      TimeTypes <- ["-tcr"];                Both => Ok(TimeTypes { modified: false, changed: false, accessed: false, created: true  }));

        // Multiples
        test!(time_uu:   TimeTypes <- ["-u", "--modified"];    Both => Ok(TimeTypes { modified: true,  changed: false, accessed: true,  created: false }));


        // Errors
        test!(time_tea:  TimeTypes <- ["--time=tea"];          Both => err OptionsError::BadArgument(&flags::TIME, OsString::from("tea")));
        test!(t_ea:      TimeTypes <- ["-tea"];                Both => err OptionsError::BadArgument(&flags::TIME, OsString::from("ea")));

        // Overriding
        test!(overridden:   TimeTypes <- ["-tcr", "-tmod"];    Last => Ok(TimeTypes { modified: true,  changed: false, accessed: false, created: false }));
        test!(overridden_2: TimeTypes <- ["-tcr", "-tmod"];    Complain => err OptionsError::Duplicate(Flag::Short(b't'), Flag::Short(b't')));
    }


    mod views {
        use super::*;

        use crate::output::grid::Options as GridOptions;


        // Default
        test!(empty:         Mode <- [], None;            Both => like Ok(Mode::Grid(_)));

        // Grid views
        test!(original_g:    Mode <- ["-G"], None;        Both => like Ok(Mode::Grid(GridOptions { across: false, .. })));
        test!(grid:          Mode <- ["--grid"], None;    Both => like Ok(Mode::Grid(GridOptions { across: false, .. })));
        test!(across:        Mode <- ["--across"], None;  Both => like Ok(Mode::Grid(GridOptions { across: true,  .. })));
        test!(gracross:      Mode <- ["-xG"], None;       Both => like Ok(Mode::Grid(GridOptions { across: true,  .. })));

        // Lines views
        test!(lines:         Mode <- ["--oneline"], None;     Both => like Ok(Mode::Lines));
        test!(prima:         Mode <- ["-1"], None;            Both => like Ok(Mode::Lines));

        // Details views
        test!(long:          Mode <- ["--long"], None;    Both => like Ok(Mode::Details(_)));
        test!(ell:           Mode <- ["-l"], None;        Both => like Ok(Mode::Details(_)));

        // Grid-details views
        test!(lid:           Mode <- ["--long", "--grid"], None;  Both => like Ok(Mode::GridDetails(_)));
        test!(leg:           Mode <- ["-lG"], None;               Both => like Ok(Mode::GridDetails(_)));

        // Options that do nothing with --long
        test!(long_across:   Mode <- ["--long", "--across"],   None;  Last => like Ok(Mode::Details(_)));

        // Options that do nothing without --long
        test!(just_header:   Mode <- ["--header"],   None;  Last => like Ok(Mode::Grid(_)));
        test!(just_group:    Mode <- ["--group"],    None;  Last => like Ok(Mode::Grid(_)));
        test!(just_inode:    Mode <- ["--inode"],    None;  Last => like Ok(Mode::Grid(_)));
        test!(just_links:    Mode <- ["--links"],    None;  Last => like Ok(Mode::Grid(_)));
        test!(just_blocks:   Mode <- ["--blocks"],   None;  Last => like Ok(Mode::Grid(_)));
        test!(just_binary:   Mode <- ["--binary"],   None;  Last => like Ok(Mode::Grid(_)));
        test!(just_bytes:    Mode <- ["--bytes"],    None;  Last => like Ok(Mode::Grid(_)));
        test!(just_numeric:  Mode <- ["--numeric"],  None;  Last => like Ok(Mode::Grid(_)));

        #[cfg(feature = "git")]
        test!(just_git:      Mode <- ["--git"],    None;  Last => like Ok(Mode::Grid(_)));

        test!(just_header_2: Mode <- ["--header"],   None;  Complain => err OptionsError::Useless(&flags::HEADER,  false, &flags::LONG));
        test!(just_group_2:  Mode <- ["--group"],    None;  Complain => err OptionsError::Useless(&flags::GROUP,   false, &flags::LONG));
        test!(just_inode_2:  Mode <- ["--inode"],    None;  Complain => err OptionsError::Useless(&flags::INODE,   false, &flags::LONG));
        test!(just_links_2:  Mode <- ["--links"],    None;  Complain => err OptionsError::Useless(&flags::LINKS,   false, &flags::LONG));
        test!(just_blocks_2: Mode <- ["--blocks"],   None;  Complain => err OptionsError::Useless(&flags::BLOCKS,  false, &flags::LONG));
        test!(just_binary_2: Mode <- ["--binary"],   None;  Complain => err OptionsError::Useless(&flags::BINARY,  false, &flags::LONG));
        test!(just_bytes_2:  Mode <- ["--bytes"],    None;  Complain => err OptionsError::Useless(&flags::BYTES,   false, &flags::LONG));
        test!(just_numeric2: Mode <- ["--numeric"],  None;  Complain => err OptionsError::Useless(&flags::NUMERIC, false, &flags::LONG));

        #[cfg(feature = "git")]
        test!(just_git_2:    Mode <- ["--git"],    None;  Complain => err OptionsError::Useless(&flags::GIT,    false, &flags::LONG));

        // Contradictions and combinations
        test!(lgo:           Mode <- ["--long", "--grid", "--oneline"], None;  Both => like Ok(Mode::Lines));
        test!(lgt:           Mode <- ["--long", "--grid", "--tree"],    None;  Both => like Ok(Mode::Details(_)));
        test!(tgl:           Mode <- ["--tree", "--grid", "--long"],    None;  Both => like Ok(Mode::GridDetails(_)));
        test!(tlg:           Mode <- ["--tree", "--long", "--grid"],    None;  Both => like Ok(Mode::GridDetails(_)));
        test!(ot:            Mode <- ["--oneline", "--tree"],           None;  Both => like Ok(Mode::Details(_)));
        test!(og:            Mode <- ["--oneline", "--grid"],           None;  Both => like Ok(Mode::Grid(_)));
        test!(tg:            Mode <- ["--tree", "--grid"],              None;  Both => like Ok(Mode::Grid(_)));
    }
}
