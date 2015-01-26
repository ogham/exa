extern crate getopts;
extern crate natord;

use file::File;
use column::{Column, SizeFormat};
use column::Column::*;
use output::View;
use term::dimensions;

use std::ascii::AsciiExt;
use std::cmp::Ordering;
use std::fmt;
use std::slice::Iter;

use self::Misfire::*;

/// The *Options* struct represents a parsed version of the user's
/// command-line options.
#[derive(PartialEq, Debug)]
pub struct Options {
    pub list_dirs: bool,
    path_strs: Vec<String>,
    reverse: bool,
    show_invisibles: bool,
    sort_field: SortField,
    view: View,
}

impl Options {

    /// Call getopts on the given slice of command-line strings.
    pub fn getopts(args: &[String]) -> Result<Options, Misfire> {
        let opts = &[
            getopts::optflag("1", "oneline",   "display one entry per line"),
            getopts::optflag("a", "all",       "show dot-files"),
            getopts::optflag("b", "binary",    "use binary prefixes in file sizes"),
            getopts::optflag("B", "bytes",     "list file sizes in bytes, without prefixes"),
            getopts::optflag("d", "list-dirs", "list directories as regular files"),
            getopts::optflag("g", "group",     "show group as well as user"),
            getopts::optflag("h", "header",    "show a header row at the top"),
            getopts::optflag("H", "links",     "show number of hard links"),
            getopts::optflag("l", "long",      "display extended details and attributes"),
            getopts::optflag("i", "inode",     "show each file's inode number"),
            getopts::optflag("r", "reverse",   "reverse order of files"),
            getopts::optopt ("s", "sort",      "field to sort by", "WORD"),
            getopts::optflag("S", "blocks",    "show number of file system blocks"),
            getopts::optflag("x", "across",    "sort multi-column view entries across"),
            getopts::optflag("?", "help",      "show list of command-line options"),
        ];

        let matches = match getopts::getopts(args, opts) {
            Ok(m) => m,
            Err(e) => return Err(Misfire::InvalidOptions(e)),
        };

        if matches.opt_present("help") {
            return Err(Misfire::Help(getopts::usage("Usage:\n  exa [options] [files...]", opts)));
        }

        let sort_field = match matches.opt_str("sort") {
            Some(word) => try!(SortField::from_word(word)),
            None => SortField::Name,
        };

        Ok(Options {
            list_dirs:       matches.opt_present("list-dirs"),
            path_strs:       if matches.free.is_empty() { vec![ ".".to_string() ] } else { matches.free.clone() },
            reverse:         matches.opt_present("reverse"),
            show_invisibles: matches.opt_present("all"),
            sort_field:      sort_field,
            view:            try!(view(&matches)),
        })
    }

    /// Iterate over the non-option arguments left oven from getopts.
    pub fn path_strings(&self) -> Iter<String> {
        self.path_strs.iter()
    }

    /// Display the files using this Option's View.
    pub fn view(&self, files: Vec<File>) {
        self.view.view(files)
    }

    /// Transform the files (sorting, reversing, filtering) before listing them.
    pub fn transform_files<'a>(&self, mut files: Vec<File<'a>>) -> Vec<File<'a>> {

        if !self.show_invisibles {
            files = files.into_iter().filter(|f| !f.is_dotfile()).collect();
        }

        match self.sort_field {
            SortField::Unsorted => {},
            SortField::Name => files.sort_by(|a, b| natord::compare(&*a.name, &*b.name)),
            SortField::Size => files.sort_by(|a, b| a.stat.size.cmp(&b.stat.size)),
            SortField::FileInode => files.sort_by(|a, b| a.stat.unstable.inode.cmp(&b.stat.unstable.inode)),
            SortField::Extension => files.sort_by(|a, b| {
                if a.ext.cmp(&b.ext) == Ordering::Equal {
                    Ordering::Equal
                }
                else {
                    a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase())
                }
            }),
        }

        if self.reverse {
            files.reverse();
        }

        files
    }
}

/// User-supplied field to sort by
#[derive(PartialEq, Debug, Copy)]
pub enum SortField {
    Unsorted, Name, Extension, Size, FileInode
}

impl SortField {

    /// Find which field to use based on a user-supplied word.
    fn from_word(word: String) -> Result<SortField, Misfire> {
        match word.as_slice() {
            "name"  => Ok(SortField::Name),
            "size"  => Ok(SortField::Size),
            "ext"   => Ok(SortField::Extension),
            "none"  => Ok(SortField::Unsorted),
            "inode" => Ok(SortField::FileInode),
            field   => Err(SortField::none(field))
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
    pub fn error_code(&self) -> isize {
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

/// Turns the Getopts results object into a View object.
fn view(matches: &getopts::Matches) -> Result<View, Misfire> {
    if matches.opt_present("long") {
        if matches.opt_present("across") {
            Err(Misfire::Useless("across", true, "long"))
        }
        else if matches.opt_present("oneline") {
            Err(Misfire::Useless("oneline", true, "long"))
        }
        else {
            Ok(View::Details(try!(columns(matches)), matches.opt_present("header")))
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
    else if matches.opt_present("oneline") {
        if matches.opt_present("across") {
            Err(Misfire::Useless("across", true, "oneline"))
        }
        else {
            Ok(View::Lines)
        }
    }
    else {
        match dimensions() {
            None => Ok(View::Lines),
            Some((width, _)) => Ok(View::Grid(matches.opt_present("across"), width)),
        }
    }
}

/// Finds out which file size the user has asked for.
fn file_size(matches: &getopts::Matches) -> Result<SizeFormat, Misfire> {
    let binary = matches.opt_present("binary");
    let bytes = matches.opt_present("bytes");

    match (binary, bytes) {
        (true,  true ) => Err(Misfire::Conflict("binary", "bytes")),
        (true,  false) => Ok(SizeFormat::BinaryBytes),
        (false, true ) => Ok(SizeFormat::JustBytes),
        (false, false) => Ok(SizeFormat::DecimalBytes),
    }
}

/// Turns the Getopts results object into a list of columns for the columns
/// view, depending on the passed-in command-line arguments.
fn columns(matches: &getopts::Matches) -> Result<Vec<Column>, Misfire> {
    let mut columns = vec![];

    if matches.opt_present("inode") {
        columns.push(Inode);
    }

    columns.push(Permissions);

    if matches.opt_present("links") {
        columns.push(HardLinks);
    }

    // Fail early here if two file size flags are given
    columns.push(FileSize(try!(file_size(matches))));

    if matches.opt_present("blocks") {
        columns.push(Blocks);
    }

    columns.push(User);

    if matches.opt_present("group") {
        columns.push(Group);
    }

    columns.push(FileName);
    Ok(columns)
}

#[cfg(test)]
mod test {
    use super::Options;
    use super::Misfire;
    use super::Misfire::*;

    use std::fmt;

    fn is_helpful(misfire: Result<Options, Misfire>) -> bool {
        match misfire {
            Err(Help(_)) => true,
            _            => false,
        }
    }

    impl fmt::Display for Options {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{:?}", self)
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
        let opts = Options::getopts(&[ "this file".to_string(), "that file".to_string() ]).unwrap();
        let args: Vec<&String> = opts.path_strings().collect();
        assert_eq!(args, vec![ &"this file".to_string(), &"that file".to_string() ])
    }

    #[test]
    fn no_args() {
        let opts = Options::getopts(&[]).unwrap();
        let args: Vec<&String> = opts.path_strings().collect();
        assert_eq!(args, vec![ &".".to_string() ])
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

}
