extern crate getopts;
extern crate natord;

use file::File;
use column::{Column, SizeFormat};
use column::Column::*;
use output::View;
use term::dimensions;

use std::ascii::AsciiExt;
use std::slice::Iter;

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

pub struct Options {
    pub list_dirs: bool,
    path_strs: Vec<String>,
    reverse: bool,
    show_invisibles: bool,
    sort_field: SortField,
    pub view: View,
}

pub enum Error {
    InvalidOptions(getopts::Fail),
    Help(String),
}

impl Options {
    pub fn getopts(args: Vec<String>) -> Result<Options, Error> {
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

        let matches = match getopts::getopts(args.tail(), opts) {
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
            view:            Options::view(&matches),
        })
    }

    pub fn path_strings(&self) -> Iter<String> {
        self.path_strs.iter()
    }

    fn view(matches: &getopts::Matches) -> View {
        if matches.opt_present("long") {
            View::Details(Options::columns(matches), matches.opt_present("header"))
        }
        else if matches.opt_present("oneline") {
            View::Lines
        }
        else {
            match dimensions() {
                None => View::Lines,
                Some((width, _)) => View::Grid(matches.opt_present("across"), width),
            }
        }
    }

    fn columns(matches: &getopts::Matches) -> Vec<Column> {
        let mut columns = vec![];

        if matches.opt_present("inode") {
            columns.push(Inode);
        }

        columns.push(Permissions);

        if matches.opt_present("links") {
            columns.push(HardLinks);
        }

		if matches.opt_present("binary") {
			columns.push(FileSize(SizeFormat::BinaryBytes))
		}
		else if matches.opt_present("bytes") {
			columns.push(FileSize(SizeFormat::JustBytes))
		}
		else {
			columns.push(FileSize(SizeFormat::DecimalBytes))
		}

        if matches.opt_present("blocks") {
            columns.push(Blocks);
        }

        columns.push(User);

        if matches.opt_present("group") {
            columns.push(Group);
        }

        columns.push(FileName);
        columns
    }

    fn should_display(&self, f: &File) -> bool {
        if self.show_invisibles {
            true
        }
        else {
            !f.name.as_slice().starts_with(".")
        }
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
}
