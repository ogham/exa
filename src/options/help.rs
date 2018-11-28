use std::fmt;

use fs::feature::xattr;
use options::flags;
use options::parser::MatchedFlags;

static OPTIONS: &str = r##"
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

FILTERING AND SORTING OPTIONS
  -a, --all                  show hidden and 'dot' files
  -d, --list-dirs            list directories like regular files
  -L, --level DEPTH          limit the depth of recursion
  -r, --reverse              reverse the sort order
  -s, --sort SORT_FIELD      which field to sort by
  --group-directories-first  list directories before other files
  -D, --only-dirs            list only directories
  -I, --ignore-glob GLOBS    glob patterns (pipe-separated) of files to ignore
  --git-ignore               Ignore files mentioned in '.gitignore'
  Valid sort fields:         name, Name, extension, Extension, size, type,
                             modified, accessed, created, inode, and none.
                             date, time, old, and new all refer to modified.
"##;

static LONG_OPTIONS: &str = r##"
LONG VIEW OPTIONS
  -b, --binary       list file sizes with binary prefixes
  -B, --bytes        list file sizes in bytes, without any prefixes
  -g, --group        list each file's group
  -h, --header       add a header row to each column
  -H, --links        list each file's number of hard links
  -i, --inode        list each file's inode number
  -m, --modified     use the modified timestamp field
  -S, --blocks       show number of file system blocks
  -t, --time FIELD   which timestamp field to list (modified, accessed, created)
  -u, --accessed     use the accessed timestamp field
  -U, --created      use the created timestamp field
  --time-style       how to format timestamps (default, iso, long-iso, full-iso)"##;

static GIT_HELP: &str = r##"  --git              list each file's Git status, if tracked"##;
static EXTENDED_HELP: &str =
    r##"  -@, --extended     list each file's extended attributes and sizes"##;

/// All the information needed to display the help text, which depends
/// on which features are enabled and whether the user only wants to
/// see one section’s help.
#[derive(PartialEq, Debug)]
pub struct HelpString {
    /// Only show the help for the long section, not all the help.
    only_long: bool,

    /// Whether the --git option should be included in the help.
    git: bool,

    /// Whether the --extended option should be included in the help.
    xattrs: bool,
}

impl HelpString {
    /// Determines how to show help, if at all, based on the user’s
    /// command-line arguments. This one works backwards from the other
    /// ‘deduce’ functions, returning Err if help needs to be shown.
    ///
    /// We don’t do any strict-mode error checking here: it’s OK to give
    /// the --help or --long flags more than once. Actually checking for
    /// errors when the user wants help is kind of petty!
    pub fn deduce(matches: &MatchedFlags) -> Result<(), HelpString> {
        if matches.count(&flags::HELP) > 0 {
            let only_long = matches.count(&flags::LONG) > 0;
            let git = cfg!(feature = "git");
            let xattrs = xattr::ENABLED;
            Err(HelpString {
                only_long,
                git,
                xattrs,
            })
        } else {
            Ok(()) // no help needs to be shown
        }
    }
}

impl fmt::Display for HelpString {
    /// Format this help options into an actual string of help
    /// text to be displayed to the user.
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "Usage:\n  exa [options] [files...]")?;

        if !self.only_long {
            write!(f, "{}", OPTIONS)?;
        }

        write!(f, "{}", LONG_OPTIONS)?;

        if self.git {
            write!(f, "\n{}", GIT_HELP)?;
        }

        if self.xattrs {
            write!(f, "\n{}", EXTENDED_HELP)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use options::Options;
    use std::ffi::OsString;

    fn os(input: &'static str) -> OsString {
        let mut os = OsString::new();
        os.push(input);
        os
    }

    #[test]
    fn help() {
        let args = [os("--help")];
        let opts = Options::parse(&args, &None);
        assert!(opts.is_err())
    }

    #[test]
    fn help_with_file() {
        let args = [os("--help"), os("me")];
        let opts = Options::parse(&args, &None);
        assert!(opts.is_err())
    }

    #[test]
    fn unhelpful() {
        let args = [];
        let opts = Options::parse(&args, &None);
        assert!(opts.is_ok()) // no help when --help isn’t passed
    }
}
