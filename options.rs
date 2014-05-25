extern crate getopts;

use file::File;
use std::cmp::lexical_ordering;
use column::{Column, Permissions, FileName, FileSize, User, Group};

pub enum SortField {
    Name, Extension, Size
}

pub struct Options {
    pub showInvisibles: bool,
    pub sortField: SortField,
    pub reverse: bool,
    pub dirs: Vec<StrBuf>,
}

impl SortField {
    pub fn from_word(word: StrBuf) -> SortField {
        match word.as_slice() {
            "name" => Name,
            "size" => Size,
            "ext" => Extension,
            _ => fail!("Invalid sorting order"),
        }
    }

    fn sort(&self, files: &mut Vec<File>) {
        match *self {
            Name => files.sort_by(|a, b| a.name.cmp(&b.name)),
            Size => files.sort_by(|a, b| a.stat.size.cmp(&b.stat.size)),
            Extension => files.sort_by(|a, b| {
                let exts = a.ext.cmp(&b.ext);
                let names = a.name.cmp(&b.name);
                lexical_ordering(exts, names)
            }),
        }
    }
}

impl Options {
    pub fn getopts(args: Vec<StrBuf>) -> Result<Options, getopts::Fail_> {
        let opts = ~[
            getopts::optflag("a", "all", "show dot-files"),
            getopts::optflag("r", "reverse", "reverse order of files"),
            getopts::optopt("s", "sort", "field to sort by", "WORD"),
        ];

        match getopts::getopts(args.tail(), opts) {
            Err(f) => Err(f),
            Ok(matches) => Ok(Options {
                showInvisibles: matches.opt_present("all"),
                reverse: matches.opt_present("reverse"),
                sortField: matches.opt_str("sort").map(|word| SortField::from_word(word)).unwrap_or(Name),
                dirs: matches.free,
            })
        }
    }

    pub fn sort(&self, files: &mut Vec<File>) {
        self.sortField.sort(files);
    }

    pub fn show(&self, f: &File) -> bool {
        if self.showInvisibles {
            true
        } else {
            !f.name.starts_with(".")
        }
    }

    pub fn columns(&self) -> ~[Column] {
        return ~[
            Permissions,
            FileSize(false),
            User,
            Group,
            FileName,
        ];
    }

}
