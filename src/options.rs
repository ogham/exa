extern crate getopts;

use file::File;
use column::Column;
use column::Column::*;
use term::dimensions;

use std::ascii::AsciiExt;

pub enum SortField {
    Unsorted, Name, Extension, Size, FileInode
}

impl SortField {
    fn from_word(word: String) -> SortField {
        match word.as_slice() {
            "name"  => SortField::Name,
            "size"  => SortField::Size,
            "ext"   => SortField::Extension,
            "none"  => SortField::Unsorted,
            "inode" => SortField::FileInode,
            _       => panic!("Invalid sorting order"),
        }
    }
}

pub enum View {
    Details(Vec<Column>),
    Lines,
    Grid(bool, uint),
}

pub struct Options {
    pub header: bool,
    pub list_dirs: bool,
    pub path_strs: Vec<String>,
    pub reverse: bool,
    pub show_invisibles: bool,
    pub sort_field: SortField,
    pub view: View,
}

impl Options {
    pub fn getopts(args: Vec<String>) -> Result<Options, int> {
        let opts = [
            getopts::optflag("1", "oneline",   "display one entry per line"),
            getopts::optflag("a", "all",       "show dot-files"),
            getopts::optflag("b", "binary",    "use binary prefixes in file sizes"),
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

        let matches = match getopts::getopts(args.tail(), &opts) {
        	Ok(m) => m,
        	Err(e) => {
        		println!("Invalid options: {}", e);
        		return Err(1);
        	}
        };

        if matches.opt_present("help") {
        	println!("exa - ls with more features\n\n{}", getopts::usage("Usage:\n  exa [options] [files...]", &opts))
        	return Err(2);
        }

        Ok(Options {
			header:          matches.opt_present("header"),
			list_dirs:       matches.opt_present("list-dirs"),
			path_strs:       if matches.free.is_empty() { vec![ ".".to_string() ] } else { matches.free.clone() },
			reverse:         matches.opt_present("reverse"),
			show_invisibles: matches.opt_present("all"),
			sort_field:      matches.opt_str("sort").map(|word| SortField::from_word(word)).unwrap_or(SortField::Name),
			view:            Options::view(&matches),
		})
    }

    fn view(matches: &getopts::Matches) -> View {
        if matches.opt_present("long") {
            View::Details(Options::columns(matches))
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

        columns.push(FileSize(matches.opt_present("binary")));

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
            SortField::Name => files.sort_by(|a, b| a.parts.cmp(&b.parts)),
            SortField::Size => files.sort_by(|a, b| a.stat.size.cmp(&b.stat.size)),
            SortField::FileInode => files.sort_by(|a, b| a.stat.unstable.inode.cmp(&b.stat.unstable.inode)),
            SortField::Extension => files.sort_by(|a, b| {
                let exts  = a.ext.clone().map(|e| e.to_ascii_lower()).cmp(&b.ext.clone().map(|e| e.to_ascii_lower()));
                let names = a.name.to_ascii_lower().cmp(&b.name.to_ascii_lower());
                exts.cmp(&names)
            }),
        }

        if self.reverse {
            files.reverse();
        }

        files
    }
}
