use std::ffi::OsStr;

use getopts;

use fs::feature::xattr;
use output::{Details, GridDetails};

mod dir_action;
pub use self::dir_action::{DirAction, RecurseOptions};

mod filter;
pub use self::filter::{FileFilter, SortField, SortCase};

mod help;
use self::help::*;

mod misfire;
pub use self::misfire::Misfire;

mod view;
pub use self::view::View;


/// These **options** represent a parsed, error-checked versions of the
/// user’s command-line options.
#[derive(PartialEq, Debug, Clone)]
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
    pub fn getopts<S>(args: &[S]) -> Result<(Options, Vec<String>), Misfire>
    where S: AsRef<OsStr> {
        let mut opts = getopts::Options::new();

        opts.optflag("v", "version",   "display version of exa");
        opts.optflag("?", "help",      "show list of command-line options");

        // Display options
        opts.optflag("1", "oneline",      "display one entry per line");
        opts.optflag("G", "grid",         "display entries in a grid view (default)");
        opts.optflag("l", "long",         "display extended details and attributes");
        opts.optflag("R", "recurse",      "recurse into directories");
        opts.optflag("T", "tree",         "recurse into subdirectories in a tree view");
        opts.optflag("x", "across",       "sort multi-column view entries across");
        opts.optflag("F", "classify",     "show file type indicator (one of */=@|)");
        opts.optopt ("",  "color",        "when to show anything in colours", "WHEN");
        opts.optopt ("",  "colour",       "when to show anything in colours (alternate spelling)", "WHEN");
        opts.optflag("",  "color-scale",  "use a colour scale when displaying file sizes (alternate spelling)");
        opts.optflag("",  "colour-scale", "use a colour scale when displaying file sizes");

        // Filtering and sorting options
        opts.optflag("",  "group-directories-first", "list directories before other files");
        opts.optflag("a", "all",         "show dot-files");
        opts.optflag("d", "list-dirs",   "list directories as regular files");
        opts.optflag("r", "reverse",     "reverse order of files");
        opts.optopt ("s", "sort",        "field to sort by", "WORD");
        opts.optopt ("I", "ignore-glob", "patterns (|-separated) of names to ignore", "GLOBS");

        // Long view options
        opts.optflag("b", "binary",    "use binary prefixes in file sizes");
        opts.optflag("B", "bytes",     "list file sizes in bytes, without prefixes");
        opts.optflag("g", "group",     "show group as well as user");
        opts.optflag("h", "header",    "show a header row at the top");
        opts.optflag("H", "links",     "show number of hard links");
        opts.optflag("i", "inode",     "show each file's inode number");
        opts.optopt ("L", "level",     "maximum depth of recursion", "DEPTH");
        opts.optflag("m", "modified",  "display timestamp of most recent modification");
        opts.optflag("S", "blocks",    "show number of file system blocks");
        opts.optopt ("t", "time",      "which timestamp to show for a file", "WORD");
        opts.optflag("u", "accessed",  "display timestamp of last access for a file");
        opts.optflag("U", "created",   "display timestamp of creation for a file");

        if cfg!(feature="git") {
            opts.optflag("", "git", "show git status");
        }

        if xattr::ENABLED {
            opts.optflag("@", "extended", "display extended attribute keys and sizes");
        }

        let matches = match opts.parse(args) {
            Ok(m)   => m,
            Err(e)  => return Err(Misfire::InvalidOptions(e)),
        };

        if matches.opt_present("help") {
            let mut help_string = "Usage:\n  exa [options] [files...]\n".to_owned();

            if !matches.opt_present("long") {
                help_string.push_str(OPTIONS);
            }

            help_string.push_str(LONG_OPTIONS);

            if cfg!(feature="git") {
                help_string.push_str(GIT_HELP);
                help_string.push('\n');
            }

            if xattr::ENABLED {
                help_string.push_str(EXTENDED_HELP);
                help_string.push('\n');
            }

            return Err(Misfire::Help(help_string));
        }
        else if matches.opt_present("version") {
            return Err(Misfire::Version);
        }

        let options = Options::deduce(&matches)?;
        Ok((options, matches.free))
    }

    /// Whether the View specified in this set of options includes a Git
    /// status column. It’s only worth trying to discover a repository if the
    /// results will end up being displayed.
    pub fn should_scan_for_git(&self) -> bool {
        match self.view {
            View::Details(Details { columns: Some(cols), .. }) => cols.should_scan_for_git(),
            View::GridDetails(GridDetails { details: Details { columns: Some(cols), .. }, .. }) => cols.should_scan_for_git(),
            _ => false,
        }
    }

    /// Determines the complete set of options based on the given command-line
    /// arguments, after they’ve been parsed.
    fn deduce(matches: &getopts::Matches) -> Result<Options, Misfire> {
        let dir_action = DirAction::deduce(matches)?;
        let filter = FileFilter::deduce(matches)?;
        let view = View::deduce(matches, filter.clone(), dir_action)?;

        Ok(Options {
            dir_action: dir_action,
            view:       view,
            filter:     filter,  // TODO: clone
        })
    }
}


#[cfg(test)]
mod test {
    use super::{Options, Misfire, SortField, SortCase};
    use fs::feature::xattr;

    fn is_helpful<T>(misfire: Result<T, Misfire>) -> bool {
        match misfire {
            Err(Misfire::Help(_)) => true,
            _                     => false,
        }
    }

    #[test]
    fn help() {
        let opts = Options::getopts(&[ "--help".to_string() ]);
        assert!(is_helpful(opts))
    }

    #[test]
    fn help_with_file() {
        let opts = Options::getopts(&[ "--help".to_string(), "me".to_string() ]);
        assert!(is_helpful(opts))
    }

    #[test]
    fn files() {
        let args = Options::getopts(&[ "this file".to_string(), "that file".to_string() ]).unwrap().1;
        assert_eq!(args, vec![ "this file".to_string(), "that file".to_string() ])
    }

    #[test]
    fn no_args() {
        let nothing: Vec<String> = Vec::new();
        let args = Options::getopts(&nothing).unwrap().1;
        assert!(args.is_empty());  // Listing the `.` directory is done in main.rs
    }

    #[test]
    fn file_sizes() {
        let opts = Options::getopts(&[ "--long", "--binary", "--bytes" ]);
        assert_eq!(opts.unwrap_err(), Misfire::Conflict("binary", "bytes"))
    }

    #[test]
    fn just_binary() {
        let opts = Options::getopts(&[ "--binary" ]);
        assert_eq!(opts.unwrap_err(), Misfire::Useless("binary", false, "long"))
    }

    #[test]
    fn just_bytes() {
        let opts = Options::getopts(&[ "--bytes" ]);
        assert_eq!(opts.unwrap_err(), Misfire::Useless("bytes", false, "long"))
    }

    #[test]
    fn long_across() {
        let opts = Options::getopts(&[ "--long", "--across" ]);
        assert_eq!(opts, Err(Misfire::Useless("across", true, "long")))
    }

    #[test]
    fn oneline_across() {
        let opts = Options::getopts(&[ "--oneline", "--across" ]);
        assert_eq!(opts, Err(Misfire::Useless("across", true, "oneline")))
    }

    #[test]
    fn just_header() {
        let opts = Options::getopts(&[ "--header" ]);
        assert_eq!(opts.unwrap_err(), Misfire::Useless("header", false, "long"))
    }

    #[test]
    fn just_group() {
        let opts = Options::getopts(&[ "--group" ]);
        assert_eq!(opts.unwrap_err(), Misfire::Useless("group", false, "long"))
    }

    #[test]
    fn just_inode() {
        let opts = Options::getopts(&[ "--inode" ]);
        assert_eq!(opts.unwrap_err(), Misfire::Useless("inode", false, "long"))
    }

    #[test]
    fn just_links() {
        let opts = Options::getopts(&[ "--links" ]);
        assert_eq!(opts.unwrap_err(), Misfire::Useless("links", false, "long"))
    }

    #[test]
    fn just_blocks() {
        let opts = Options::getopts(&[ "--blocks" ]);
        assert_eq!(opts.unwrap_err(), Misfire::Useless("blocks", false, "long"))
    }

    #[test]
    fn test_sort_size() {
        let opts = Options::getopts(&[ "--sort=size" ]);
        assert_eq!(opts.unwrap().0.filter.sort_field, SortField::Size);
    }

    #[test]
    fn test_sort_name() {
        let opts = Options::getopts(&[ "--sort=name" ]);
        assert_eq!(opts.unwrap().0.filter.sort_field, SortField::Name(SortCase::Sensitive));
    }

    #[test]
    fn test_sort_name_lowercase() {
        let opts = Options::getopts(&[ "--sort=Name" ]);
        assert_eq!(opts.unwrap().0.filter.sort_field, SortField::Name(SortCase::Insensitive));
    }

    #[test]
    #[cfg(feature="git")]
    fn just_git() {
        let opts = Options::getopts(&[ "--git" ]);
        assert_eq!(opts.unwrap_err(), Misfire::Useless("git", false, "long"))
    }

    #[test]
    fn extended_without_long() {
        if xattr::ENABLED {
            let opts = Options::getopts(&[ "--extended" ]);
            assert_eq!(opts.unwrap_err(), Misfire::Useless("extended", false, "long"))
        }
    }

    #[test]
    fn level_without_recurse_or_tree() {
        let opts = Options::getopts(&[ "--level", "69105" ]);
        assert_eq!(opts.unwrap_err(), Misfire::Useless2("level", "recurse", "tree"))
    }
}
