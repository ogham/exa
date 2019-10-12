use output::{View, Mode, grid, details, lines};
use output::grid_details::{self, RowThreshold};
use output::table::{TimeTypes, Environment, SizeFormat, Columns, Options as TableOptions};
use output::time::TimeFormat;

use options::{flags, Misfire, Vars};
use options::parser::MatchedFlags;

use fs::PlatformMetadata;
use fs::feature::xattr;


impl View {

    /// Determine which view to use and all of that view’s arguments.
    pub fn deduce<V: Vars>(matches: &MatchedFlags, vars: &V) -> Result<View, Misfire> {
        use options::style::Styles;

        let mode = Mode::deduce(matches, vars)?;
        let Styles { colours, style } = Styles::deduce(matches, vars, || *TERM_WIDTH)?;
        Ok(View { mode, colours, style })
    }
}


impl Mode {

    /// Determine the mode from the command-line arguments.
    pub fn deduce<V: Vars>(matches: &MatchedFlags, vars: &V) -> Result<Mode, Misfire> {
        use options::misfire::Misfire::*;

        let long = || {
            if matches.has(&flags::ACROSS)? && !matches.has(&flags::GRID)? {
                Err(Useless(&flags::ACROSS, true, &flags::LONG))
            }
            else if matches.has(&flags::ONE_LINE)? {
                Err(Useless(&flags::ONE_LINE, true, &flags::LONG))
            }
            else {
                Ok(details::Options {
                    table: Some(TableOptions::deduce(matches, vars)?),
                    header: matches.has(&flags::HEADER)?,
                    xattr: xattr::ENABLED && matches.has(&flags::EXTENDED)?,
                    icons: matches.has(&flags::ICONS)?,
                })
            }
        };

        let other_options_scan = || {
            if let Some(width) = TerminalWidth::deduce(vars)?.width() {
                if matches.has(&flags::ONE_LINE)? {
                    if matches.has(&flags::ACROSS)? {
                        Err(Useless(&flags::ACROSS, true, &flags::ONE_LINE))
                    }
                    else {
                        let lines = lines::Options { icons: matches.has(&flags::ICONS)? };
                        Ok(Mode::Lines(lines))
                    }
                }
                else if matches.has(&flags::TREE)? {
                    let details = details::Options {
                        table: None,
                        header: false,
                        xattr: xattr::ENABLED && matches.has(&flags::EXTENDED)?,
                        icons: matches.has(&flags::ICONS)?,
                    };

                    Ok(Mode::Details(details))
                }
                else {
                    let grid = grid::Options {
                        across: matches.has(&flags::ACROSS)?,
                        console_width: width,
                        icons: matches.has(&flags::ICONS)?,
                    };

                    Ok(Mode::Grid(grid))
                }
            }

            // If the terminal width couldn’t be matched for some reason, such
            // as the program’s stdout being connected to a file, then
            // fallback to the lines or details view.
            else if matches.has(&flags::TREE)? {
                let details = details::Options {
                    table: None,
                    header: false,
                    xattr: xattr::ENABLED && matches.has(&flags::EXTENDED)?,
                    icons: matches.has(&flags::ICONS)?,
                };

                Ok(Mode::Details(details))
            }
            else if matches.has(&flags::LONG)? {
                let details = long()?;
                Ok(Mode::Details(details))
            } else {
                let lines = lines::Options { icons: matches.has(&flags::ICONS)?, };
                Ok(Mode::Lines(lines))
            }
        };

        if matches.has(&flags::LONG)? {
            let details = long()?;
            if matches.has(&flags::GRID)? {
                let other_options_mode = other_options_scan()?;
                if let Mode::Grid(grid) = other_options_mode {
                    let row_threshold = RowThreshold::deduce(vars)?;
                    return Ok(Mode::GridDetails(grid_details::Options { grid, details, row_threshold }));
                }
                else {
                    return Ok(other_options_mode);
                }
            }
            else {
                return Ok(Mode::Details(details));
            }
        }

        // If --long hasn’t been passed, then check if we need to warn the
        // user about flags that won’t have any effect.
        if matches.is_strict() {
            for option in &[ &flags::BINARY, &flags::BYTES, &flags::INODE, &flags::LINKS,
                             &flags::HEADER, &flags::BLOCKS, &flags::TIME, &flags::GROUP ] {
                if matches.has(option)? {
                    return Err(Useless(*option, false, &flags::LONG));
                }
            }

            if cfg!(feature="git") && matches.has(&flags::GIT)? {
                return Err(Useless(&flags::GIT, false, &flags::LONG));
            }
            else if matches.has(&flags::LEVEL)? && !matches.has(&flags::RECURSE)? && !matches.has(&flags::TREE)? {
                // TODO: I'm not sure if the code even gets this far.
                // There is an identical check in dir_action
                return Err(Useless2(&flags::LEVEL, &flags::RECURSE, &flags::TREE));
            }
        }

        other_options_scan()
    }
}


/// The width of the terminal requested by the user.
#[derive(PartialEq, Debug)]
enum TerminalWidth {

    /// The user requested this specific number of columns.
    Set(usize),

    /// The terminal was found to have this number of columns.
    Terminal(usize),

    /// The user didn’t request any particular terminal width.
    Unset,
}

impl TerminalWidth {

    /// Determine a requested terminal width from the command-line arguments.
    ///
    /// Returns an error if a requested width doesn’t parse to an integer.
    fn deduce<V: Vars>(vars: &V) -> Result<TerminalWidth, Misfire> {
        use options::vars;

        if let Some(columns) = vars.get(vars::COLUMNS).and_then(|s| s.into_string().ok()) {
            match columns.parse() {
                Ok(width)  => Ok(TerminalWidth::Set(width)),
                Err(e)     => Err(Misfire::FailedParse(e)),
            }
        }
        else if let Some(width) = *TERM_WIDTH {
            Ok(TerminalWidth::Terminal(width))
        }
        else {
            Ok(TerminalWidth::Unset)
        }
    }

    fn width(&self) -> Option<usize> {
        match *self {
            TerminalWidth::Set(width)       |
            TerminalWidth::Terminal(width)  => Some(width),
            TerminalWidth::Unset            => None,
        }
    }
}


impl RowThreshold {

    /// Determine whether to use a row threshold based on the given
    /// environment variables.
    fn deduce<V: Vars>(vars: &V) -> Result<RowThreshold, Misfire> {
        use options::vars;

        if let Some(columns) = vars.get(vars::EXA_GRID_ROWS).and_then(|s| s.into_string().ok()) {
            match columns.parse() {
                Ok(rows)  => Ok(RowThreshold::MinimumRows(rows)),
                Err(e)    => Err(Misfire::FailedParse(e)),
            }
        }
        else {
            Ok(RowThreshold::AlwaysGrid)
        }
    }
}


impl TableOptions {
    fn deduce<V: Vars>(matches: &MatchedFlags, vars: &V) -> Result<Self, Misfire> {
        let env = Environment::load_all();
        let time_format = TimeFormat::deduce(matches, vars)?;
        let size_format = SizeFormat::deduce(matches)?;
        let extra_columns = Columns::deduce(matches)?;
        Ok(TableOptions { env, time_format, size_format, extra_columns })
    }
}


impl Columns {
    fn deduce(matches: &MatchedFlags) -> Result<Self, Misfire> {
        let time_types = TimeTypes::deduce(matches)?;
        let git = cfg!(feature="git") && matches.has(&flags::GIT)?;

        let blocks = matches.has(&flags::BLOCKS)?;
        let group  = matches.has(&flags::GROUP)?;
        let inode  = matches.has(&flags::INODE)?;
        let links  = matches.has(&flags::LINKS)?;

        Ok(Columns { time_types, git, blocks, group, inode, links })
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
    fn deduce(matches: &MatchedFlags) -> Result<SizeFormat, Misfire> {
        let flag = matches.has_where(|f| f.matches(&flags::BINARY) || f.matches(&flags::BYTES))?;

        Ok(match flag {
            Some(f) if f.matches(&flags::BINARY)  => SizeFormat::BinaryBytes,
            Some(f) if f.matches(&flags::BYTES)   => SizeFormat::JustBytes,
            _                                     => SizeFormat::DecimalBytes,
        })
    }
}


impl TimeFormat {

    /// Determine how time should be formatted in timestamp columns.
    fn deduce<V: Vars>(matches: &MatchedFlags, vars: &V) -> Result<TimeFormat, Misfire> {
        pub use output::time::{DefaultFormat, ISOFormat};

        let word = match matches.get(&flags::TIME_STYLE)? {
            Some(w) => w.to_os_string(),
            None    => {
                use options::vars;
                match vars.get(vars::TIME_STYLE) {
                    Some(ref t) if !t.is_empty() => t.clone(),
                    _                            => return Ok(TimeFormat::DefaultFormat(DefaultFormat::load()))
                }
            },
        };

        if &word == "default" {
            Ok(TimeFormat::DefaultFormat(DefaultFormat::load()))
        }
        else if &word == "iso" {
            Ok(TimeFormat::ISOFormat(ISOFormat::load()))
        }
        else if &word == "long-iso" {
            Ok(TimeFormat::LongISO)
        }
        else if &word == "full-iso" {
            Ok(TimeFormat::FullISO)
        }
        else {
            Err(Misfire::BadArgument(&flags::TIME_STYLE, word.into()))
        }
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
    fn deduce(matches: &MatchedFlags) -> Result<TimeTypes, Misfire> {
        let possible_word = matches.get(&flags::TIME)?;
        let modified = matches.has(&flags::MODIFIED)?;
        let changed  = matches.has(&flags::CHANGED)?;
        let accessed = matches.has(&flags::ACCESSED)?;
        let created  = matches.has(&flags::CREATED)?;

        let time_types = if let Some(word) = possible_word {
            if modified {
                return Err(Misfire::Useless(&flags::MODIFIED, true, &flags::TIME));
            }
            else if changed {
                return Err(Misfire::Useless(&flags::CHANGED, true, &flags::TIME));
            }
            else if accessed {
                return Err(Misfire::Useless(&flags::ACCESSED, true, &flags::TIME));
            }
            else if created {
                return Err(Misfire::Useless(&flags::CREATED, true, &flags::TIME));
            }
            else if word == "mod" || word == "modified" {
                TimeTypes { modified: true,  changed: false, accessed: false, created: false }
            }
            else if word == "ch" || word == "changed" {
                TimeTypes { modified: false, changed: true,  accessed: false, created: false }
            }
            else if word == "acc" || word == "accessed" {
                TimeTypes { modified: false, changed: false, accessed: true,  created: false }
            }
            else if word == "cr" || word == "created" {
                TimeTypes { modified: false, changed: false, accessed: false, created: true  }
            }
            else {
                return Err(Misfire::BadArgument(&flags::TIME, word.into()));
            }
        }
        else if modified || changed || accessed || created {
            TimeTypes { modified, changed, accessed, created }
        }
        else {
            TimeTypes::default()
        };

        let mut fields = vec![];
        if time_types.modified { fields.push(PlatformMetadata::ModifiedTime); }
        if time_types.changed  { fields.push(PlatformMetadata::ChangedTime); }
        if time_types.accessed { fields.push(PlatformMetadata::AccessedTime); }
        if time_types.created  { fields.push(PlatformMetadata::CreatedTime); }

        for field in fields {
            if let Err(misfire) = field.check_supported() {
                return Err(misfire);
            }
        }
        Ok(time_types)
    }
}


// Gets, then caches, the width of the terminal that exa is running in.
// This gets used multiple times above, with no real guarantee of order,
// so it’s easier to just cache it the first time it runs.
lazy_static! {
    static ref TERM_WIDTH: Option<usize> = {
        // All of stdin, stdout, and stderr could not be connected to a
        // terminal, but we’re only interested in stdout because it’s
        // where the output goes.
        use term_size::dimensions_stdout;
        dimensions_stdout().map(|t| t.0)
    };
}



#[cfg(test)]
mod test {
    use super::*;
    use std::ffi::OsString;
    use options::flags;
    use options::parser::{Flag, Arg};

    use options::test::parse_for_test;
    use options::test::Strictnesses::*;

    static TEST_ARGS: &[&Arg] = &[ &flags::BINARY, &flags::BYTES,    &flags::TIME_STYLE,
                                   &flags::TIME,   &flags::MODIFIED, &flags::CHANGED,
                                   &flags::CREATED, &flags::ACCESSED, &flags::ICONS,
                                   &flags::HEADER, &flags::GROUP,  &flags::INODE, &flags::GIT,
                                   &flags::LINKS,  &flags::BLOCKS, &flags::LONG,  &flags::LEVEL,
                                   &flags::GRID,   &flags::ACROSS, &flags::ONE_LINE ];

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

        test!(both_5:  SizeFormat <- ["--binary", "--binary"];  Complain => err Misfire::Duplicate(Flag::Long("binary"), Flag::Long("binary")));
        test!(both_6:  SizeFormat <- ["--bytes",  "--binary"];  Complain => err Misfire::Duplicate(Flag::Long("bytes"),  Flag::Long("binary")));
        test!(both_7:  SizeFormat <- ["--binary", "--bytes"];   Complain => err Misfire::Duplicate(Flag::Long("binary"), Flag::Long("bytes")));
        test!(both_8:  SizeFormat <- ["--bytes",  "--bytes"];   Complain => err Misfire::Duplicate(Flag::Long("bytes"),  Flag::Long("bytes")));
    }


    mod time_formats {
        use super::*;

        // These tests use pattern matching because TimeFormat doesn’t
        // implement PartialEq.

        // Default behaviour
        test!(empty:     TimeFormat <- [], None;                            Both => like Ok(TimeFormat::DefaultFormat(_)));

        // Individual settings
        test!(default:   TimeFormat <- ["--time-style=default"], None;      Both => like Ok(TimeFormat::DefaultFormat(_)));
        test!(iso:       TimeFormat <- ["--time-style", "iso"], None;       Both => like Ok(TimeFormat::ISOFormat(_)));
        test!(long_iso:  TimeFormat <- ["--time-style=long-iso"], None;     Both => like Ok(TimeFormat::LongISO));
        test!(full_iso:  TimeFormat <- ["--time-style", "full-iso"], None;  Both => like Ok(TimeFormat::FullISO));

        // Overriding
        test!(actually:  TimeFormat <- ["--time-style=default", "--time-style", "iso"], None;  Last => like Ok(TimeFormat::ISOFormat(_)));
        test!(actual_2:  TimeFormat <- ["--time-style=default", "--time-style", "iso"], None;  Complain => err Misfire::Duplicate(Flag::Long("time-style"), Flag::Long("time-style")));

        test!(nevermind: TimeFormat <- ["--time-style", "long-iso", "--time-style=full-iso"], None;  Last => like Ok(TimeFormat::FullISO));
        test!(nevermore: TimeFormat <- ["--time-style", "long-iso", "--time-style=full-iso"], None;  Complain => err Misfire::Duplicate(Flag::Long("time-style"), Flag::Long("time-style")));

        // Errors
        test!(daily:     TimeFormat <- ["--time-style=24-hour"], None;  Both => err Misfire::BadArgument(&flags::TIME_STYLE, OsString::from("24-hour")));

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
        test!(t_ch:    TimeTypes <- ["-t", "ch"];            Both => Ok(TimeTypes { modified: false, changed: true,  accessed: false, created: false }));

        // Accessed
        test!(acc:       TimeTypes <- ["--accessed"];          Both => Ok(TimeTypes { modified: false, changed: false, accessed: true,  created: false }));
        test!(a:         TimeTypes <- ["-u"];                  Both => Ok(TimeTypes { modified: false, changed: false, accessed: true,  created: false }));
        test!(time_acc:  TimeTypes <- ["--time", "accessed"];  Both => Ok(TimeTypes { modified: false, changed: false, accessed: true,  created: false }));
        test!(time_a:    TimeTypes <- ["-t", "acc"];           Both => Ok(TimeTypes { modified: false, changed: false, accessed: true,  created: false }));

        // Created
        #[cfg(not(target_os = "linux"))]
        test!(cr:        TimeTypes <- ["--created"];           Both => Ok(TimeTypes { modified: false, changed: false, accessed: false, created: true  }));
        #[cfg(target_os = "linux")]
        test!(cr:        TimeTypes <- ["--created"];           Both => err Misfire::Unsupported("creation time is not available on this platform currently".to_string()));
        #[cfg(not(target_os = "linux"))]
        test!(c:         TimeTypes <- ["-U"];                  Both => Ok(TimeTypes { modified: false, changed: false, accessed: false, created: true  }));
        #[cfg(not(target_os = "linux"))]
        test!(time_cr:   TimeTypes <- ["--time=created"];      Both => Ok(TimeTypes { modified: false, changed: false, accessed: false, created: true  }));
        #[cfg(not(target_os = "linux"))]
        test!(t_cr:      TimeTypes <- ["-tcr"];                Both => Ok(TimeTypes { modified: false, changed: false, accessed: false, created: true  }));

        // Multiples
        test!(time_uu:   TimeTypes <- ["-u", "--modified"];    Both => Ok(TimeTypes { modified: true, changed: false, accessed: true,  created: false }));


        // Errors
        test!(time_tea:  TimeTypes <- ["--time=tea"];          Both => err Misfire::BadArgument(&flags::TIME, OsString::from("tea")));
        test!(t_ea:      TimeTypes <- ["-tea"];                Both => err Misfire::BadArgument(&flags::TIME, OsString::from("ea")));

        // Overriding
        test!(overridden:   TimeTypes <- ["-tcr", "-tmod"];    Last => Ok(TimeTypes { modified: true,  changed: false, accessed: false, created: false }));
        test!(overridden_2: TimeTypes <- ["-tcr", "-tmod"];    Complain => err Misfire::Duplicate(Flag::Short(b't'), Flag::Short(b't')));
    }


    mod views {
        use super::*;
        use output::grid::Options as GridOptions;
        use output::lines::Options as LineOptions;

        // Default
        test!(empty:         Mode <- [], None;            Both => like Ok(Mode::Grid(_)));

        // Grid views
        test!(original_g:    Mode <- ["-G"], None;        Both => like Ok(Mode::Grid(GridOptions { across: false, console_width: _, icons: _ })));
        test!(grid:          Mode <- ["--grid"], None;    Both => like Ok(Mode::Grid(GridOptions { across: false, console_width: _, icons: _ })));
        test!(across:        Mode <- ["--across"], None;  Both => like Ok(Mode::Grid(GridOptions { across: true,  console_width: _, icons: _ })));
        test!(gracross:      Mode <- ["-xG"], None;       Both => like Ok(Mode::Grid(GridOptions { across: true,  console_width: _, icons: _ })));
        test!(icons:         Mode <- ["--icons"], None;   Both => like Ok(Mode::Grid(GridOptions { across: _, console_width: _, icons: true})));

        // Lines views
        test!(lines:         Mode <- ["--oneline"], None; Both => like Ok(Mode::Lines(LineOptions{ icons: _ })));
        test!(prima:         Mode <- ["-1"], None;        Both => like Ok(Mode::Lines(LineOptions{ icons: _ })));
        test!(line_icon:     Mode <- ["-1", "--icons"], None; Both => like Ok(Mode::Lines(LineOptions { icons: true })));

        // Details views
        test!(long:          Mode <- ["--long"], None;    Both => like Ok(Mode::Details(_)));
        test!(ell:           Mode <- ["-l"], None;        Both => like Ok(Mode::Details(_)));

        // Grid-details views
        test!(lid:           Mode <- ["--long", "--grid"], None;  Both => like Ok(Mode::GridDetails(_)));
        test!(leg:           Mode <- ["-lG"], None;               Both => like Ok(Mode::GridDetails(_)));


        // Options that do nothing without --long
        test!(just_header:   Mode <- ["--header"], None;  Last => like Ok(Mode::Grid(_)));
        test!(just_group:    Mode <- ["--group"],  None;  Last => like Ok(Mode::Grid(_)));
        test!(just_inode:    Mode <- ["--inode"],  None;  Last => like Ok(Mode::Grid(_)));
        test!(just_links:    Mode <- ["--links"],  None;  Last => like Ok(Mode::Grid(_)));
        test!(just_blocks:   Mode <- ["--blocks"], None;  Last => like Ok(Mode::Grid(_)));
        test!(just_binary:   Mode <- ["--binary"], None;  Last => like Ok(Mode::Grid(_)));
        test!(just_bytes:    Mode <- ["--bytes"],  None;  Last => like Ok(Mode::Grid(_)));

        #[cfg(feature="git")]
        test!(just_git:      Mode <- ["--git"],    None;  Last => like Ok(Mode::Grid(_)));

        test!(just_header_2: Mode <- ["--header"], None;  Complain => err Misfire::Useless(&flags::HEADER, false, &flags::LONG));
        test!(just_group_2:  Mode <- ["--group"],  None;  Complain => err Misfire::Useless(&flags::GROUP,  false, &flags::LONG));
        test!(just_inode_2:  Mode <- ["--inode"],  None;  Complain => err Misfire::Useless(&flags::INODE,  false, &flags::LONG));
        test!(just_links_2:  Mode <- ["--links"],  None;  Complain => err Misfire::Useless(&flags::LINKS,  false, &flags::LONG));
        test!(just_blocks_2: Mode <- ["--blocks"], None;  Complain => err Misfire::Useless(&flags::BLOCKS, false, &flags::LONG));
        test!(just_binary_2: Mode <- ["--binary"], None;  Complain => err Misfire::Useless(&flags::BINARY, false, &flags::LONG));
        test!(just_bytes_2:  Mode <- ["--bytes"],  None;  Complain => err Misfire::Useless(&flags::BYTES,  false, &flags::LONG));

        #[cfg(feature="git")]
        test!(just_git_2:    Mode <- ["--git"],    None;  Complain => err Misfire::Useless(&flags::GIT,    false, &flags::LONG));
    }
}
