use dir::Dir;
use file::File;
use column::Column;
use column::Column::*;
use output::{Grid, Details};
use term::dimensions;

use std::cmp::Ordering;
use std::fmt;

use getopts;
use natord;

use datetime::local::{LocalDateTime, DatePiece};

use self::Misfire::*;

/// The *Options* struct represents a parsed version of the user's
/// command-line options.
#[derive(PartialEq, Debug, Copy)]
pub struct Options {
    pub dir_action: DirAction,
    pub filter: FileFilter,
    pub view: View,
}

#[derive(PartialEq, Debug, Copy)]
pub struct FileFilter {
    reverse: bool,
    show_invisibles: bool,
    sort_field: SortField,
}

#[derive(PartialEq, Debug, Copy)]
pub enum View {
    Details(Details),
    Lines,
    Grid(Grid),
}

impl Options {

    /// Call getopts on the given slice of command-line strings.
    pub fn getopts(args: &[String]) -> Result<(Options, Vec<String>), Misfire> {
        let mut opts = getopts::Options::new();
        opts.optflag("1", "oneline",   "display one entry per line");
        opts.optflag("a", "all",       "show dot-files");
        opts.optflag("b", "binary",    "use binary prefixes in file sizes");
        opts.optflag("B", "bytes",     "list file sizes in bytes, without prefixes");
        opts.optflag("d", "list-dirs", "list directories as regular files");
        opts.optflag("g", "group",     "show group as well as user");
        opts.optflag("h", "header",    "show a header row at the top");
        opts.optflag("H", "links",     "show number of hard links");
        opts.optflag("i", "inode",     "show each file's inode number");
        opts.optflag("l", "long",      "display extended details and attributes");
        opts.optflag("m", "modified",  "display timestamp of most recent modification");
        opts.optflag("r", "reverse",   "reverse order of files");
        opts.optflag("R", "recurse",   "recurse into directories");
        opts.optopt ("s", "sort",      "field to sort by", "WORD");
        opts.optflag("S", "blocks",    "show number of file system blocks");
        opts.optopt ("t", "time",      "which timestamp to show for a file", "WORD");
        opts.optflag("T", "tree",      "recurse into subdirectories in a tree view");
        opts.optflag("u", "accessed",  "display timestamp of last access for a file");
        opts.optflag("U", "created",   "display timestamp of creation for a file");
        opts.optflag("x", "across",    "sort multi-column view entries across");
        opts.optflag("?", "help",      "show list of command-line options");

        let matches = match opts.parse(args) {
            Ok(m) => m,
            Err(e) => return Err(Misfire::InvalidOptions(e)),
        };

        if matches.opt_present("help") {
            return Err(Misfire::Help(opts.usage("Usage:\n  exa [options] [files...]")));
        }

        let sort_field = match matches.opt_str("sort") {
            Some(word) => try!(SortField::from_word(word)),
            None => SortField::Name,
        };

        let filter = FileFilter {
            reverse:         matches.opt_present("reverse"),
            show_invisibles: matches.opt_present("all"),
            sort_field:      sort_field,
        };

        let path_strs = if matches.free.is_empty() {
            vec![ ".".to_string() ]
        }
        else {
            matches.free.clone()
        };

        Ok((Options {
            dir_action: try!(DirAction::deduce(&matches)),
            view:       try!(View::deduce(&matches, filter)),
            filter:     filter,
        }, path_strs))
    }

    pub fn transform_files<'a>(&self, files: &mut Vec<File<'a>>) {
        self.filter.transform_files(files)
    }
}

impl FileFilter {
    /// Transform the files (sorting, reversing, filtering) before listing them.
    pub fn transform_files<'a>(&self, files: &mut Vec<File<'a>>) {

        if !self.show_invisibles {
            files.retain(|f| !f.is_dotfile());
        }

        match self.sort_field {
            SortField::Unsorted => {},
            SortField::Name => files.sort_by(|a, b| natord::compare(&*a.name, &*b.name)),
            SortField::Size => files.sort_by(|a, b| a.stat.size.cmp(&b.stat.size)),
            SortField::FileInode => files.sort_by(|a, b| a.stat.unstable.inode.cmp(&b.stat.unstable.inode)),
            SortField::Extension => files.sort_by(|a, b| match a.ext.cmp(&b.ext) {
                Ordering::Equal => natord::compare(&*a.name, &*b.name),
                order => order
            }),
            SortField::ModifiedDate => files.sort_by(|a, b| a.stat.modified.cmp(&b.stat.modified)),
            SortField::AccessedDate => files.sort_by(|a, b| a.stat.accessed.cmp(&b.stat.accessed)),
            SortField::CreatedDate  => files.sort_by(|a, b| a.stat.created.cmp(&b.stat.created)),
        }

        if self.reverse {
            files.reverse();
        }
    }
}

/// User-supplied field to sort by.
#[derive(PartialEq, Debug, Copy)]
pub enum SortField {
    Unsorted, Name, Extension, Size, FileInode,
    ModifiedDate, AccessedDate, CreatedDate,
}

impl SortField {

    /// Find which field to use based on a user-supplied word.
    fn from_word(word: String) -> Result<SortField, Misfire> {
        match word.as_slice() {
            "name" | "filename"  => Ok(SortField::Name),
            "size" | "filesize"  => Ok(SortField::Size),
            "ext"  | "extension" => Ok(SortField::Extension),
            "mod"  | "modified"  => Ok(SortField::ModifiedDate),
            "acc"  | "accessed"  => Ok(SortField::AccessedDate),
            "cr"   | "created"   => Ok(SortField::CreatedDate),
            "none"               => Ok(SortField::Unsorted),
            "inode"              => Ok(SortField::FileInode),
            field                => Err(SortField::none(field))
        }
    }

    /// How to display an error when the word didn't match with anything.
    fn none(field: &str) -> Misfire {
        Misfire::InvalidOptions(getopts::Fail::UnrecognizedOption(format!("--sort {}", field)))
    }
}

/// One of these things could happen instead of listing files.
#[derive(PartialEq, Debug)]
pub enum Misfire {

    /// The getopts crate didn't like these arguments.
    InvalidOptions(getopts::Fail),

    /// The user asked for help. This isn't strictly an error, which is why
    /// this enum isn't named Error!
    Help(String),

    /// Two options were given that conflict with one another
    Conflict(&'static str, &'static str),

    /// An option was given that does nothing when another one either is or
    /// isn't present.
    Useless(&'static str, bool, &'static str),
}

impl Misfire {
    /// The OS return code this misfire should signify.
    pub fn error_code(&self) -> i32 {
        if let Help(_) = *self { 2 }
                          else { 3 }
    }
}

impl fmt::Display for Misfire {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InvalidOptions(ref e) => write!(f, "{}", e),
            Help(ref text)        => write!(f, "{}", text),
            Conflict(a, b)        => write!(f, "Option --{} conflicts with option {}.", a, b),
            Useless(a, false, b)  => write!(f, "Option --{} is useless without option --{}.", a, b),
            Useless(a, true, b)   => write!(f, "Option --{} is useless given option --{}.", a, b),
        }
    }
}

impl View {
    pub fn deduce(matches: &getopts::Matches, filter: FileFilter) -> Result<View, Misfire> {
        if matches.opt_present("long") {
            if matches.opt_present("across") {
                Err(Misfire::Useless("across", true, "long"))
            }
            else if matches.opt_present("oneline") {
                Err(Misfire::Useless("oneline", true, "long"))
            }
            else {
                let details = Details {
                        columns: try!(Columns::deduce(matches)),
                        header: matches.opt_present("header"),
                        tree: matches.opt_present("recurse"),
                        filter: filter,
                };

                Ok(View::Details(details))
            }
        }
        else if matches.opt_present("binary") {
            Err(Misfire::Useless("binary", false, "long"))
        }
        else if matches.opt_present("bytes") {
            Err(Misfire::Useless("bytes", false, "long"))
        }
        else if matches.opt_present("inode") {
            Err(Misfire::Useless("inode", false, "long"))
        }
        else if matches.opt_present("links") {
            Err(Misfire::Useless("links", false, "long"))
        }
        else if matches.opt_present("header") {
            Err(Misfire::Useless("header", false, "long"))
        }
        else if matches.opt_present("blocks") {
            Err(Misfire::Useless("blocks", false, "long"))
        }
        else if matches.opt_present("time") {
            Err(Misfire::Useless("time", false, "long"))
        }
        else if matches.opt_present("tree") {
            Err(Misfire::Useless("tree", false, "long"))
        }
        else if matches.opt_present("oneline") {
            if matches.opt_present("across") {
                Err(Misfire::Useless("across", true, "oneline"))
            }
            else {
                Ok(View::Lines)
            }
        }
        else {
            if let Some((width, _)) = dimensions() {
                let grid = Grid {
                    across: matches.opt_present("across"),
                    console_width: width
                };

                Ok(View::Grid(grid))
            }
            else {
                // If the terminal width couldn't be matched for some reason, such
                // as the program's stdout being connected to a file, then
                // fallback to the lines view.
                Ok(View::Lines)
            }
        }
    }
}

#[derive(PartialEq, Debug, Copy)]
pub enum SizeFormat {
    DecimalBytes,
    BinaryBytes,
    JustBytes,
}

impl SizeFormat {
    pub fn deduce(matches: &getopts::Matches) -> Result<SizeFormat, Misfire> {
        let binary = matches.opt_present("binary");
        let bytes  = matches.opt_present("bytes");

        match (binary, bytes) {
            (true,  true ) => Err(Misfire::Conflict("binary", "bytes")),
            (true,  false) => Ok(SizeFormat::BinaryBytes),
            (false, true ) => Ok(SizeFormat::JustBytes),
            (false, false) => Ok(SizeFormat::DecimalBytes),
        }
    }
}

#[derive(PartialEq, Debug, Copy)]
pub enum TimeType {
    FileAccessed,
    FileModified,
    FileCreated,
}

impl TimeType {
    pub fn header(&self) -> &'static str {
        match *self {
            TimeType::FileAccessed => "Date Accessed",
            TimeType::FileModified => "Date Modified",
            TimeType::FileCreated  => "Date Created",
        }
    }
}

#[derive(PartialEq, Debug, Copy)]
pub struct TimeTypes {
    accessed: bool,
    modified: bool,
    created:  bool,
}

impl TimeTypes {

    /// Find which field to use based on a user-supplied word.
    fn deduce(matches: &getopts::Matches) -> Result<TimeTypes, Misfire> {
        let possible_word = matches.opt_str("time");
        let modified = matches.opt_present("modified");
        let created  = matches.opt_present("created");
        let accessed = matches.opt_present("accessed");

        if let Some(word) = possible_word {
            if modified {
                return Err(Misfire::Useless("modified", true, "time"));
            }
            else if created {
                return Err(Misfire::Useless("created", true, "time"));
            }
            else if accessed {
                return Err(Misfire::Useless("accessed", true, "time"));
            }

            match word.as_slice() {
                "mod" | "modified"  => Ok(TimeTypes { accessed: false, modified: true, created: false }),
                "acc" | "accessed"  => Ok(TimeTypes { accessed: true, modified: false, created: false }),
                "cr"  | "created"   => Ok(TimeTypes { accessed: false, modified: false, created: true }),
                field   => Err(TimeTypes::none(field)),
            }
        }
        else {
            if modified || created || accessed {
                Ok(TimeTypes { accessed: accessed, modified: modified, created: created })
            }
            else {
                Ok(TimeTypes { accessed: false, modified: true, created: false })
            }
        }
    }

    /// How to display an error when the word didn't match with anything.
    fn none(field: &str) -> Misfire {
        Misfire::InvalidOptions(getopts::Fail::UnrecognizedOption(format!("--time {}", field)))
    }
}

/// What to do when encountering a directory?
#[derive(PartialEq, Debug, Copy)]
pub enum DirAction {
    AsFile, List, Recurse, Tree
}

impl DirAction {
    pub fn deduce(matches: &getopts::Matches) -> Result<DirAction, Misfire> {
        let recurse = matches.opt_present("recurse");
        let list    = matches.opt_present("list-dirs");
        let tree    = matches.opt_present("tree");

        match (recurse, list, tree) {
            (false, _,     true ) => Err(Misfire::Useless("tree", false, "recurse")),
            (true,  true,  _    ) => Err(Misfire::Conflict("recurse", "list-dirs")),
            (true,  false, false) => Ok(DirAction::Recurse),
            (true,  false, true ) => Ok(DirAction::Tree),
            (false, true,  _    ) => Ok(DirAction::AsFile),
            (false, false, _    ) => Ok(DirAction::List),
        }
    }
}

#[derive(PartialEq, Copy, Debug)]
pub struct Columns {
    size_format: SizeFormat,
    time_types: TimeTypes,
    inode: bool,
    links: bool,
    blocks: bool,
    group: bool,
}

impl Columns {
    pub fn deduce(matches: &getopts::Matches) -> Result<Columns, Misfire> {
        Ok(Columns {
            size_format: try!(SizeFormat::deduce(matches)),
            time_types:  try!(TimeTypes::deduce(matches)),
            inode:  matches.opt_present("inode"),
            links:  matches.opt_present("links"),
            blocks: matches.opt_present("blocks"),
            group:  matches.opt_present("group"),
        })
    }

    pub fn for_dir(&self, dir: Option<&Dir>) -> Vec<Column> {
        let mut columns = vec![];

        if self.inode {
            columns.push(Inode);
        }

        columns.push(Permissions);

        if self.links {
            columns.push(HardLinks);
        }

        columns.push(FileSize(self.size_format));

        if self.blocks {
            columns.push(Blocks);
        }

        columns.push(User);

        if self.group {
            columns.push(Group);
        }

        let current_year = LocalDateTime::now().year();

        if self.time_types.modified {
            columns.push(Timestamp(TimeType::FileModified, current_year));
        }

        if self.time_types.created {
            columns.push(Timestamp(TimeType::FileCreated, current_year));
        }

        if self.time_types.accessed {
            columns.push(Timestamp(TimeType::FileAccessed, current_year));
        }

        if cfg!(feature="git") {
            if let Some(d) = dir {
                if d.has_git_repo() {
                    columns.push(GitStatus);
                }
            }
        }

        columns
    }
}

#[cfg(test)]
mod test {
    use super::Options;
    use super::Misfire;
    use super::Misfire::*;

    fn is_helpful<T>(misfire: Result<T, Misfire>) -> bool {
        match misfire {
            Err(Help(_)) => true,
            _            => false,
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
        let args = Options::getopts(&[]).unwrap().1;
        assert_eq!(args, vec![ ".".to_string() ])
    }

    #[test]
    fn file_sizes() {
        let opts = Options::getopts(&[ "--long".to_string(), "--binary".to_string(), "--bytes".to_string() ]);
        assert_eq!(opts.unwrap_err(), Misfire::Conflict("binary", "bytes"))
    }

    #[test]
    fn just_binary() {
        let opts = Options::getopts(&[ "--binary".to_string() ]);
        assert_eq!(opts.unwrap_err(), Misfire::Useless("binary", false, "long"))
    }

    #[test]
    fn just_bytes() {
        let opts = Options::getopts(&[ "--bytes".to_string() ]);
        assert_eq!(opts.unwrap_err(), Misfire::Useless("bytes", false, "long"))
    }

    #[test]
    fn long_across() {
        let opts = Options::getopts(&[ "--long".to_string(), "--across".to_string() ]);
        assert_eq!(opts.unwrap_err(), Misfire::Useless("across", true, "long"))
    }

    #[test]
    fn oneline_across() {
        let opts = Options::getopts(&[ "--oneline".to_string(), "--across".to_string() ]);
        assert_eq!(opts.unwrap_err(), Misfire::Useless("across", true, "oneline"))
    }

    #[test]
    fn just_header() {
        let opts = Options::getopts(&[ "--header".to_string() ]);
        assert_eq!(opts.unwrap_err(), Misfire::Useless("header", false, "long"))
    }

    #[test]
    fn just_inode() {
        let opts = Options::getopts(&[ "--inode".to_string() ]);
        assert_eq!(opts.unwrap_err(), Misfire::Useless("inode", false, "long"))
    }

    #[test]
    fn just_links() {
        let opts = Options::getopts(&[ "--links".to_string() ]);
        assert_eq!(opts.unwrap_err(), Misfire::Useless("links", false, "long"))
    }

    #[test]
    fn just_blocks() {
        let opts = Options::getopts(&[ "--blocks".to_string() ]);
        assert_eq!(opts.unwrap_err(), Misfire::Useless("blocks", false, "long"))
    }

    #[test]
    fn tree_without_recurse() {
        let opts = Options::getopts(&[ "--tree".to_string() ]);
        assert_eq!(opts.unwrap_err(), Misfire::Useless("tree", false, "recurse"))
    }

}
