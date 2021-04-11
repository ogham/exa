use std::fmt;

use crate::fs::feature::xattr;
use crate::options::flags;
use crate::options::parser::MatchedFlags;


static USAGE_PART1: &str = "Usage:
  exa [options] [files...]

META OPTIONS
  -?, --help         show list of command-line options
  -v, --version      show version of exa

DISPLAY OPTIONS
  -1, --oneline      display one entry per line
  -l, --long         display extended file metadata as a table
  -G, --grid         display entries as a grid (default)
  -x, --across       sort the grid across, rather than downwards
  -R, --recurse      recurse into directories
  -T, --tree         recurse into directories as a tree
  -F, --classify     display type indicator by file names
  --colo[u]r=WHEN    when to use terminal colours (always, auto, never)
  --colo[u]r-scale   highlight levels of file sizes distinctly
  --icons            display icons
  --no-icons         don't display icons (always overrides --icons)

FILTERING AND SORTING OPTIONS
  -a, --all                  show hidden and 'dot' files
  -d, --list-dirs            list directories as files; don't list their contents
  -L, --level DEPTH          limit the depth of recursion
  -r, --reverse              reverse the sort order
  -s, --sort SORT_FIELD      which field to sort by
  --group-directories-first  list directories before other files
  -D, --only-dirs            list only directories
  -I, --ignore-glob GLOBS    glob patterns (pipe-separated) of files to ignore";

  static USAGE_PART2: &str = "  \
  Valid sort fields:         name, Name, extension, Extension, size, type,
                             modified, accessed, created, inode, and none.
                             date, time, old, and new all refer to modified.

LONG VIEW OPTIONS
  -b, --binary         list file sizes with binary prefixes
  -B, --bytes          list file sizes in bytes, without any prefixes
  -g, --group          list each file's group
  -h, --header         add a header row to each column
  -H, --links          list each file's number of hard links
  -i, --inode          list each file's inode number
  -m, --modified       use the modified timestamp field
  -n, --numeric        list numeric user and group IDs
  -S, --blocks         show number of file system blocks
  -t, --time FIELD     which timestamp field to list (modified, accessed, created)
  -u, --accessed       use the accessed timestamp field
  -U, --created        use the created timestamp field
  --changed            use the changed timestamp field
  --time-style         how to format timestamps (default, iso, long-iso, full-iso)
  --no-permissions     suppress the permissions field
  --octal-permissions  list each file's permission in octal format
  --no-filesize        suppress the filesize field
  --no-user            suppress the user field
  --no-time            suppress the time field";

static GIT_FILTER_HELP: &str = "  --git-ignore               ignore files mentioned in '.gitignore'";
static GIT_VIEW_HELP:   &str = "  --git                list each file's Git status, if tracked or ignored";
static EXTENDED_HELP:   &str = "  -@, --extended       list each file's extended attributes and sizes";


/// All the information needed to display the help text, which depends
/// on which features are enabled and whether the user only wants to
/// see one section’s help.
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct HelpString;

impl HelpString {

    /// Determines how to show help, if at all, based on the user’s
    /// command-line arguments. This one works backwards from the other
    /// ‘deduce’ functions, returning Err if help needs to be shown.
    ///
    /// We don’t do any strict-mode error checking here: it’s OK to give
    /// the --help or --long flags more than once. Actually checking for
    /// errors when the user wants help is kind of petty!
    pub fn deduce(matches: &MatchedFlags<'_>) -> Option<Self> {
        if matches.count(&flags::HELP) > 0 {
            Some(Self)
        }
        else {
            None
        }
    }
}

impl fmt::Display for HelpString {

    /// Format this help options into an actual string of help
    /// text to be displayed to the user.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", USAGE_PART1)?;

        if cfg!(feature = "git") {
            write!(f, "\n{}", GIT_FILTER_HELP)?;
        }

        write!(f, "\n{}", USAGE_PART2)?;

        if cfg!(feature = "git") {
            write!(f, "\n{}", GIT_VIEW_HELP)?;
        }

        if xattr::ENABLED {
            write!(f, "\n{}", EXTENDED_HELP)?;
        }

        writeln!(f)
    }
}


#[cfg(test)]
mod test {
    use crate::options::{Options, OptionsResult};
    use std::ffi::OsStr;

    #[test]
    fn help() {
        let args = vec![ OsStr::new("--help") ];
        let opts = Options::parse(args, &None);
        assert!(matches!(opts, OptionsResult::Help(_)));
    }

    #[test]
    fn help_with_file() {
        let args = vec![ OsStr::new("--help"), OsStr::new("me") ];
        let opts = Options::parse(args, &None);
        assert!(matches!(opts, OptionsResult::Help(_)));
    }

    #[test]
    fn unhelpful() {
        let args = vec![];
        let opts = Options::parse(args, &None);
        assert!(! matches!(opts, OptionsResult::Help(_)))  // no help when --help isn’t passed
    }
}
