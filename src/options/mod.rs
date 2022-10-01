//! Parsing command-line strings into exa options.
//!
//! This module imports exa’s configuration types, such as `View` (the details
//! of displaying multiple files) and `DirAction` (what to do when encountering
//! a directory), and implements `deduce` methods on them so they can be
//! configured using command-line options.
//!
//!
//! ## Useless and overridden options
//!
//! Let’s say exa was invoked with just one argument: `exa --inode`. The
//! `--inode` option is used in the details view, where it adds the inode
//! column to the output. But because the details view is *only* activated with
//! the `--long` argument, adding `--inode` without it would not have any
//! effect.
//!
//! For a long time, exa’s philosophy was that the user should be warned
//! whenever they could be mistaken like this. If you tell exa to display the
//! inode, and it *doesn’t* display the inode, isn’t that more annoying than
//! having it throw an error back at you?
//!
//! However, this doesn’t take into account *configuration*. Say a user wants
//! to configure exa so that it lists inodes in the details view, but otherwise
//! functions normally. A common way to do this for command-line programs is to
//! define a shell alias that specifies the details they want to use every
//! time. For the inode column, the alias would be:
//!
//! `alias exa="exa --inode"`
//!
//! Using this alias means that although the inode column will be shown in the
//! details view, you’re now *only* allowed to use the details view, as any
//! other view type will result in an error. Oops!
//!
//! Another example is when an option is specified twice, such as `exa
//! --sort=Name --sort=size`. Did the user change their mind about sorting, and
//! accidentally specify the option twice?
//!
//! Again, exa rejected this case, throwing an error back to the user instead
//! of trying to guess how they want their output sorted. And again, this
//! doesn’t take into account aliases being used to set defaults. A user who
//! wants their files to be sorted case-insensitively may configure their shell
//! with the following:
//!
//! `alias exa="exa --sort=Name"`
//!
//! Just like the earlier example, the user now can’t use any other sort order,
//! because exa refuses to guess which one they meant. It’s *more* annoying to
//! have to go back and edit the command than if there were no error.
//!
//! Fortunately, there’s a heuristic for telling which options came from an
//! alias and which came from the actual command-line: aliased options are
//! nearer the beginning of the options array, and command-line options are
//! nearer the end. This means that after the options have been parsed, exa
//! needs to traverse them *backwards* to find the last-most-specified one.
//!
//! For example, invoking exa with `exa --sort=size` when that alias is present
//! would result in a full command-line of:
//!
//! `exa --sort=Name --sort=size`
//!
//! `--sort=size` should override `--sort=Name` because it’s closer to the end
//! of the arguments array. In fact, because there’s no way to tell where the
//! arguments came from — it’s just a heuristic — this will still work even
//! if no aliases are being used!
//!
//! Finally, this isn’t just useful when options could override each other.
//! Creating an alias `exal="exa --long --inode --header"` then invoking `exal
//! --grid --long` shouldn’t complain about `--long` being given twice when
//! it’s clear what the user wants.


use std::ffi::OsStr;

use crate::fs::dir_action::DirAction;
use crate::fs::filter::{FileFilter, GitIgnore};
use crate::output::{View, Mode, details, grid_details};
use crate::theme::Options as ThemeOptions;

mod dir_action;
mod file_name;
mod filter;
mod flags;
mod theme;
mod view;

mod error;
pub use self::error::{OptionsError, NumberSource};

mod help;
use self::help::HelpString;

mod parser;
use self::parser::MatchedFlags;

pub mod vars;
pub use self::vars::Vars;

mod version;
use self::version::VersionString;


/// These **options** represent a parsed, error-checked versions of the
/// user’s command-line options.
#[derive(Debug)]
pub struct Options {

    /// The action to perform when encountering a directory rather than a
    /// regular file.
    pub dir_action: DirAction,

    /// How to sort and filter files before outputting them.
    pub filter: FileFilter,

    /// The user’s preference of view to use (lines, grid, details, or
    /// grid-details) along with the options on how to render file names.
    /// If the view requires the terminal to have a width, and there is no
    /// width, then the view will be downgraded.
    pub view: View,

    /// The options to make up the styles of the UI and file names.
    pub theme: ThemeOptions,
}

impl Options {

    /// Parse the given iterator of command-line strings into an Options
    /// struct and a list of free filenames, using the environment variables
    /// for extra options.
    #[allow(unused_results)]
    pub fn parse<'args, I, V>(args: I, vars: &V) -> OptionsResult<'args>
    where I: IntoIterator<Item = &'args OsStr>,
          V: Vars,
    {
        use crate::options::parser::{Matches, Strictness};

        let strictness = match vars.get(vars::EXA_STRICT) {
            None                         => Strictness::UseLastArguments,
            Some(ref t) if t.is_empty()  => Strictness::UseLastArguments,
            Some(_)                      => Strictness::ComplainAboutRedundantArguments,
        };

        let Matches { flags, frees } = match flags::ALL_ARGS.parse(args, strictness) {
            Ok(m)    => m,
            Err(pe)  => return OptionsResult::InvalidOptions(OptionsError::Parse(pe)),
        };

        if let Some(help) = HelpString::deduce(&flags) {
            return OptionsResult::Help(help);
        }

        if let Some(version) = VersionString::deduce(&flags) {
            return OptionsResult::Version(version);
        }

        match Self::deduce(&flags, vars) {
            Ok(options)  => OptionsResult::Ok(options, frees),
            Err(oe)      => OptionsResult::InvalidOptions(oe),
        }
    }

    /// Whether the View specified in this set of options includes a Git
    /// status column. It’s only worth trying to discover a repository if the
    /// results will end up being displayed.
    pub fn should_scan_for_git(&self) -> bool {
        if self.filter.git_ignore == GitIgnore::CheckAndIgnore {
            return true;
        }

        match self.view.mode {
            Mode::Details(details::Options { table: Some(ref table), .. }) |
            Mode::GridDetails(grid_details::Options { details: details::Options { table: Some(ref table), .. }, .. }) => table.columns.git,
            _ => false,
        }
    }

    /// Determines the complete set of options based on the given command-line
    /// arguments, after they’ve been parsed.
    fn deduce<V: Vars>(matches: &MatchedFlags<'_>, vars: &V) -> Result<Self, OptionsError> {
        if cfg!(not(feature = "git")) &&
                matches.has_where_any(|f| f.matches(&flags::GIT) || f.matches(&flags::GIT_IGNORE)).is_some() {
            return Err(OptionsError::Unsupported(String::from(
                "Options --git and --git-ignore can't be used because `git` feature was disabled in this build of exa"
            )));
        }

        let view = View::deduce(matches, vars)?;
        let dir_action = DirAction::deduce(matches, matches!(view.mode, Mode::Details(_)))?;
        let filter = FileFilter::deduce(matches)?;
        let theme = ThemeOptions::deduce(matches, vars)?;

        Ok(Self { dir_action, filter, view, theme })
    }
}


/// The result of the `Options::getopts` function.
#[derive(Debug)]
pub enum OptionsResult<'args> {

    /// The options were parsed successfully.
    Ok(Options, Vec<&'args OsStr>),

    /// There was an error parsing the arguments.
    InvalidOptions(OptionsError),

    /// One of the arguments was `--help`, so display help.
    Help(HelpString),

    /// One of the arguments was `--version`, so display the version number.
    Version(VersionString),
}


#[cfg(test)]
pub mod test {
    use crate::options::parser::{Arg, MatchedFlags};
    use std::ffi::OsStr;

    #[derive(PartialEq, Eq, Debug)]
    pub enum Strictnesses {
        Last,
        Complain,
        Both,
    }

    /// This function gets used by the other testing modules.
    /// It can run with one or both strictness values: if told to run with
    /// both, then both should resolve to the same result.
    ///
    /// It returns a vector with one or two elements in.
    /// These elements can then be tested with `assert_eq` or what have you.
    pub fn parse_for_test<T, F>(inputs: &[&str], args: &'static [&'static Arg], strictnesses: Strictnesses, get: F) -> Vec<T>
    where F: Fn(&MatchedFlags<'_>) -> T
    {
        use self::Strictnesses::*;
        use crate::options::parser::{Args, Strictness};

        let bits = inputs.iter().map(OsStr::new).collect::<Vec<_>>();
        let mut result = Vec::new();

        if strictnesses == Last || strictnesses == Both {
            let results = Args(args).parse(bits.clone(), Strictness::UseLastArguments);
            result.push(get(&results.unwrap().flags));
        }

        if strictnesses == Complain || strictnesses == Both {
            let results = Args(args).parse(bits, Strictness::ComplainAboutRedundantArguments);
            result.push(get(&results.unwrap().flags));
        }

        result
    }
}
