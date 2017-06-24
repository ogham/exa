use std::ffi::OsStr;

use getopts;

use fs::feature::xattr;
use output::{Details, GridDetails};

mod dir_action;
pub use self::dir_action::{DirAction, RecurseOptions};

mod filter;
pub use self::filter::{FileFilter, SortField, SortCase};

mod help;
use self::help::HelpString;

mod misfire;
pub use self::misfire::Misfire;

mod view;
pub use self::view::{View, Mode};


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

    // Even though the arguments go in as OsStrings, they come out
    // as Strings. Invalid UTF-8 won’t be parsed, but it won’t make
    // exa core dump either.
    //
    // https://github.com/rust-lang-nursery/getopts/pull/29

    /// Call getopts on the given slice of command-line strings.
    #[allow(unused_results)]
    pub fn getopts<C>(args: C) -> Result<(Options, Vec<String>), Misfire>
    where C: IntoIterator, C::Item: AsRef<OsStr> {
        let mut opts = getopts::Options::new();

        opts.optflag("v", "version",   "show version of exa");
        opts.optflag("?", "help",      "show list of command-line options");

        // Display options
        opts.optflag("1", "oneline",      "display one entry per line");
        opts.optflag("l", "long",         "display extended file metadata in a table");
        opts.optflag("G", "grid",         "display entries as a grid (default)");
        opts.optflag("x", "across",       "sort the grid across, rather than downwards");
        opts.optflag("R", "recurse",      "recurse into directories");
        opts.optflag("T", "tree",         "recurse into directories as a tree");
        opts.optflag("F", "classify",     "display type indicator by file names (one of */=@|)");
        opts.optopt ("",  "color",        "when to use terminal colours", "WHEN");
        opts.optopt ("",  "colour",       "when to use terminal colours", "WHEN");
        opts.optflag("",  "color-scale",  "highlight levels of file sizes distinctly");
        opts.optflag("",  "colour-scale", "highlight levels of file sizes distinctly");

        // Filtering and sorting options
        opts.optflag("",  "group-directories-first", "sort directories before other files");
        opts.optflag("a", "all",         "don't hide hidden and 'dot' files");
        opts.optflag("d", "list-dirs",   "list directories like regular files");
        opts.optopt ("L", "level",       "limit the depth of recursion", "DEPTH");
        opts.optflag("r", "reverse",     "reverse the sert order");
        opts.optopt ("s", "sort",        "which field to sort by", "WORD");
        opts.optopt ("I", "ignore-glob", "ignore files that match these glob patterns", "GLOB1|GLOB2...");

        // Long view options
        opts.optflag("b", "binary",    "list file sizes with binary prefixes");
        opts.optflag("B", "bytes",     "list file sizes in bytes, without prefixes");
        opts.optflag("g", "group",     "list each file's group");
        opts.optflag("h", "header",    "add a header row to each column");
        opts.optflag("H", "links",     "list each file's number of hard links");
        opts.optflag("i", "inode",     "list each file's inode number");
        opts.optflag("m", "modified",  "use the modified timestamp field");
        opts.optflag("S", "blocks",    "list each file's number of file system blocks");
        opts.optopt ("t", "time",      "which timestamp field to show", "WORD");
        opts.optflag("u", "accessed",  "use the accessed timestamp field");
        opts.optflag("U", "created",   "use the created timestamp field");

        if cfg!(feature="git") {
            opts.optflag("", "git", "list each file's git status");
        }

        if xattr::ENABLED {
            opts.optflag("@", "extended", "list each file's extended attribute keys and sizes");
        }

        let matches = match opts.parse(args) {
            Ok(m)   => m,
            Err(e)  => return Err(Misfire::InvalidOptions(e)),
        };

        if matches.opt_present("help") {
            let help = HelpString {
                only_long: matches.opt_present("long"),
                git: cfg!(feature="git"),
                xattrs: xattr::ENABLED,
            };

            return Err(Misfire::Help(help));
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
        match self.view.mode {
            Mode::Details(Details { columns: Some(cols), .. }) |
            Mode::GridDetails(GridDetails { details: Details { columns: Some(cols), .. }, .. }) => cols.should_scan_for_git(),
            _ => false,
        }
    }

    /// Determines the complete set of options based on the given command-line
    /// arguments, after they’ve been parsed.
    fn deduce(matches: &getopts::Matches) -> Result<Options, Misfire> {
        let dir_action = DirAction::deduce(matches)?;
        let filter = FileFilter::deduce(matches)?;
        let view = View::deduce(matches, filter.clone(), dir_action)?;

        Ok(Options { dir_action, view, filter })
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
