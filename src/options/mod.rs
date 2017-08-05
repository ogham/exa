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
//! arguments came from -- it’s just a heuristic -- this will still work even
//! if no aliases are being used!
//!
//! Finally, this isn’t just useful when options could override each other.
//! Creating an alias `exal=”exa --long --inode --header”` then invoking `exal
//! --grid --long` shouldn’t complain about `--long` being given twice when
//! it’s clear what the user wants.


use std::ffi::{OsStr, OsString};

use fs::dir_action::DirAction;
use fs::filter::FileFilter;
use output::{View, Mode};
use output::details;

mod dir_action;
mod filter;
mod view;

mod help;
use self::help::HelpString;

mod misfire;
pub use self::misfire::Misfire;

mod parser;
mod flags;
use self::parser::MatchedFlags;


/// These **options** represent a parsed, error-checked versions of the
/// user’s command-line options.
#[derive(Debug)]
pub struct Options {

    /// The action to perform when encountering a directory rather than a
    /// regular file.
    pub dir_action: DirAction,

    /// How to sort and filter files before outputting them.
    pub filter: FileFilter,

    /// The type of output to use (lines, grid, or details).
    pub view: View,
}

impl Options {

    /// Call getopts on the given slice of command-line strings.
    #[allow(unused_results)]
    pub fn getopts<'args, I>(args: I) -> Result<(Options, Vec<&'args OsStr>), Misfire>
    where I: IntoIterator<Item=&'args OsString> {
        use options::parser::Matches;

        let Matches { flags, frees } = match flags::ALL_ARGS.parse(args) {
            Ok(m)   => m,
            Err(e)  => return Err(Misfire::InvalidOptions(e)),
        };

        HelpString::deduce(&flags).map_err(Misfire::Help)?;

        if flags.has(&flags::VERSION) {
            return Err(Misfire::Version);
        }

        let options = Options::deduce(&flags)?;
        Ok((options, frees))
    }

    /// Whether the View specified in this set of options includes a Git
    /// status column. It’s only worth trying to discover a repository if the
    /// results will end up being displayed.
    pub fn should_scan_for_git(&self) -> bool {
        match self.view.mode {
            Mode::Details(details::Options { table: Some(ref table), .. }) |
            Mode::GridDetails(_, details::Options { table: Some(ref table), .. }) => table.should_scan_for_git(),
            _ => false,
        }
    }

    /// Determines the complete set of options based on the given command-line
    /// arguments, after they’ve been parsed.
    fn deduce(matches: &MatchedFlags) -> Result<Options, Misfire> {
        let dir_action = DirAction::deduce(matches)?;
        let filter = FileFilter::deduce(matches)?;
        let view = View::deduce(matches)?;

        Ok(Options { dir_action, view, filter })
    }
}



#[cfg(test)]
mod test {
    use super::{Options, Misfire, flags};
    use std::ffi::OsString;
    use fs::filter::{SortField, SortCase};
    use fs::feature::xattr;

    /// Creates an `OSStr` (used in tests)
    #[cfg(test)]
    fn os(input: &'static str) -> OsString {
        let mut os = OsString::new();
        os.push(input);
        os
    }

    #[test]
    fn files() {
        let args = [ os("this file"), os("that file") ];
        let outs = Options::getopts(&args).unwrap().1;
        assert_eq!(outs, vec![ &os("this file"), &os("that file") ])
    }

    #[test]
    fn no_args() {
        let nothing: Vec<OsString> = Vec::new();
        let outs = Options::getopts(&nothing).unwrap().1;
        assert!(outs.is_empty());  // Listing the `.` directory is done in main.rs
    }

    #[test]
    fn just_binary() {
        let args = [ os("--binary") ];
        let opts = Options::getopts(&args);
        assert_eq!(opts.unwrap_err(), Misfire::Useless(&flags::BINARY, false, &flags::LONG))
    }

    #[test]
    fn just_bytes() {
        let args = [ os("--bytes") ];
        let opts = Options::getopts(&args);
        assert_eq!(opts.unwrap_err(), Misfire::Useless(&flags::BYTES, false, &flags::LONG))
    }

    #[test]
    fn long_across() {
        let args = [ os("--long"), os("--across") ];
        let opts = Options::getopts(&args);
        assert_eq!(opts.unwrap_err(), Misfire::Useless(&flags::ACROSS, true, &flags::LONG))
    }

    #[test]
    fn oneline_across() {
        let args = [ os("--oneline"), os("--across") ];
        let opts = Options::getopts(&args);
        assert_eq!(opts.unwrap_err(), Misfire::Useless(&flags::ACROSS, true, &flags::ONE_LINE))
    }

    #[test]
    fn just_header() {
        let args = [ os("--header") ];
        let opts = Options::getopts(&args);
        assert_eq!(opts.unwrap_err(), Misfire::Useless(&flags::HEADER, false, &flags::LONG))
    }

    #[test]
    fn just_group() {
        let args = [ os("--group") ];
        let opts = Options::getopts(&args);
        assert_eq!(opts.unwrap_err(), Misfire::Useless(&flags::GROUP, false, &flags::LONG))
    }

    #[test]
    fn just_inode() {
        let args = [ os("--inode") ];
        let opts = Options::getopts(&args);
        assert_eq!(opts.unwrap_err(), Misfire::Useless(&flags::INODE, false, &flags::LONG))
    }

    #[test]
    fn just_links() {
        let args = [ os("--links") ];
        let opts = Options::getopts(&args);
        assert_eq!(opts.unwrap_err(), Misfire::Useless(&flags::LINKS, false, &flags::LONG))
    }

    #[test]
    fn just_blocks() {
        let args = [ os("--blocks") ];
        let opts = Options::getopts(&args);
        assert_eq!(opts.unwrap_err(), Misfire::Useless(&flags::BLOCKS, false, &flags::LONG))
    }

    #[test]
    fn test_sort_size() {
        let args = [ os("--sort=size") ];
        let opts = Options::getopts(&args);
        assert_eq!(opts.unwrap().0.filter.sort_field, SortField::Size);
    }

    #[test]
    fn test_sort_name() {
        let args = [ os("--sort=name") ];
        let opts = Options::getopts(&args);
        assert_eq!(opts.unwrap().0.filter.sort_field, SortField::Name(SortCase::Sensitive));
    }

    #[test]
    fn test_sort_name_lowercase() {
        let args = [ os("--sort=Name") ];
        let opts = Options::getopts(&args);
        assert_eq!(opts.unwrap().0.filter.sort_field, SortField::Name(SortCase::Insensitive));
    }

    #[test]
    #[cfg(feature="git")]
    fn just_git() {
        let args = [ os("--git") ];
        let opts = Options::getopts(&args);
        assert_eq!(opts.unwrap_err(), Misfire::Useless(&flags::GIT, false, &flags::LONG))
    }

    #[test]
    fn extended_without_long() {
        if xattr::ENABLED {
            let args = [ os("--extended") ];
            let opts = Options::getopts(&args);
            assert_eq!(opts.unwrap_err(), Misfire::Useless(&flags::EXTENDED, false, &flags::LONG))
        }
    }
}
