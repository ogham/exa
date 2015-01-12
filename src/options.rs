extern crate getopts;
extern crate natord;

use file::File;
use column::{Column, SizeFormat};
use column::Column::*;
use output::View;
use term::dimensions;

use std::ascii::AsciiExt;
use std::slice::Iter;

#[derive(PartialEq, Show)]
pub enum SortField {
    Unsorted, Name, Extension, Size, FileInode
}

impl Copy for SortField { }

impl SortField {
    fn from_word(word: String) -> Result<SortField, Error> {
        match word.as_slice() {
            "name"  => Ok(SortField::Name),
            "size"  => Ok(SortField::Size),
            "ext"   => Ok(SortField::Extension),
            "none"  => Ok(SortField::Unsorted),
            "inode" => Ok(SortField::FileInode),
            field   => Err(no_sort_field(field))
        }
    }
}

fn no_sort_field(field: &str) -> Error {
    Error::InvalidOptions(getopts::Fail::UnrecognizedOption(format!("--sort {}", field)))
}

#[derive(PartialEq, Show)]
pub struct Options {
    pub list_dirs: bool,
    pub path_strs: Vec<String>,
    reverse: bool,
    show_invisibles: bool,
    sort_field: SortField,
    pub view: View,
}

#[derive(PartialEq, Show)]
pub enum Error {
    InvalidOptions(getopts::Fail),
    Help(String),
    Conflict(&'static str, &'static str),
    Useless(&'static str, bool, &'static str),
}

impl Options {
    pub fn getopts(args: &[String]) -> Result<Options, Error> {
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
            Err(e) => return Err(Error::InvalidOptions(e)),
        };

        if matches.opt_present("help") {
            return Err(Error::Help(getopts::usage("Usage:\n  exa [options] [files...]", opts)));
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

    pub fn path_strings(&self) -> Iter<String> {
        self.path_strs.iter()
    }

    pub fn transform_files<'a>(&self, unordered_files: Vec<File<'a>>) -> Vec<File<'a>> {
        let mut files: Vec<File<'a>> = unordered_files.into_iter()
            .filter(|f| self.should_display(f))
            .collect();

        match self.sort_field {
            SortField::Unsorted => {},
            SortField::Name => files.sort_by(|a, b| natord::compare(a.name.as_slice(), b.name.as_slice())),
            SortField::Size => files.sort_by(|a, b| a.stat.size.cmp(&b.stat.size)),
            SortField::FileInode => files.sort_by(|a, b| a.stat.unstable.inode.cmp(&b.stat.unstable.inode)),
            SortField::Extension => files.sort_by(|a, b| {
                let exts  = a.ext.clone().map(|e| e.to_ascii_lowercase()).cmp(&b.ext.clone().map(|e| e.to_ascii_lowercase()));
                let names = a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase());
                exts.cmp(&names)
            }),
        }

        if self.reverse {
            files.reverse();
        }

        files
    }

    fn should_display(&self, f: &File) -> bool {
        if self.show_invisibles {
            true
        }
        else {
            !f.name.as_slice().starts_with(".")
        }
    }
}

fn view(matches: &getopts::Matches) -> Result<View, Error> {
    if matches.opt_present("long") {
        if matches.opt_present("across") {
            Err(Error::Useless("across", true, "long"))
        }
        else if matches.opt_present("oneline") {
            Err(Error::Useless("across", true, "long"))
        }
        else {
            Ok(View::Details(try!(columns(matches)), matches.opt_present("header")))
        }
    }
    else if matches.opt_present("binary") {
        Err(Error::Useless("binary", false, "long"))
    }
    else if matches.opt_present("bytes") {
        Err(Error::Useless("bytes", false, "long"))
    }
    else if matches.opt_present("oneline") {
        if matches.opt_present("across") {
            Err(Error::Useless("across", true, "oneline"))
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

fn file_size(matches: &getopts::Matches) -> Result<SizeFormat, Error> {
    let binary = matches.opt_present("binary");
    let bytes = matches.opt_present("bytes");

    match (binary, bytes) {
        (true,  true ) => Err(Error::Conflict("binary", "bytes")),
        (true,  false) => Ok(SizeFormat::BinaryBytes),
        (false, true ) => Ok(SizeFormat::JustBytes),
        (false, false) => Ok(SizeFormat::DecimalBytes),
    }
}

fn columns(matches: &getopts::Matches) -> Result<Vec<Column>, Error> {
    let mut columns = vec![];

    if matches.opt_present("inode") {
        columns.push(Inode);
    }

    columns.push(Permissions);

    if matches.opt_present("links") {
        columns.push(HardLinks);
    }

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
    use super::Error;
    use super::Error::*;

    fn is_helpful(error: Result<Options, Error>) -> bool {
        match error {
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
        let opts = Options::getopts(&[ "this file".to_string(), "that file".to_string() ]);
        assert_eq!(opts.unwrap().path_strs, vec![ "this file".to_string(), "that file".to_string() ])
    }

    #[test]
    fn no_args() {
        let opts = Options::getopts(&[]);
        assert_eq!(opts.unwrap().path_strs, vec![ ".".to_string() ])
    }

    #[test]
    fn file_sizes() {
        let opts = Options::getopts(&[ "--long".to_string(), "--binary".to_string(), "--bytes".to_string() ]);
        assert_eq!(opts.unwrap_err(), Error::Conflict("binary", "bytes"))
    }

    #[test]
    fn just_binary() {
        let opts = Options::getopts(&[ "--binary".to_string() ]);
        assert_eq!(opts.unwrap_err(), Error::Useless("binary", false, "long"))
    }

    #[test]
    fn just_bytes() {
        let opts = Options::getopts(&[ "--bytes".to_string() ]);
        assert_eq!(opts.unwrap_err(), Error::Useless("bytes", false, "long"))
    }

    #[test]
    fn long_across() {
        let opts = Options::getopts(&[ "--long".to_string(), "--across".to_string() ]);
        assert_eq!(opts.unwrap_err(), Error::Useless("across", true, "long"))
    }

    #[test]
    fn oneline_across() {
        let opts = Options::getopts(&[ "--oneline".to_string(), "--across".to_string() ]);
        assert_eq!(opts.unwrap_err(), Error::Useless("across", true, "oneline"))
    }


}
