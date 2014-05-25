#![feature(phase)]
extern crate regex;
#[phase(syntax)] extern crate regex_macros;

extern crate getopts;
use std::os;
use std::io::fs;

use file::File;
use column::defaultColumns;
use options::{Options, SortField, Name};

pub mod colours;
pub mod column;
pub mod format;
pub mod file;
pub mod unix;
pub mod options;

fn main() {
    let args: Vec<StrBuf> = os::args().iter()
        .map(|x| x.to_strbuf())
        .collect();

    let opts = ~[
        getopts::optflag("a", "all", "show dot-files"),
        getopts::optflag("r", "reverse", "reverse order of files"),
        getopts::optopt("s", "sort", "field to sort by", "WORD"),
    ];

    let matches = match getopts::getopts(args.tail(), opts) {
        Ok(m) => m,
        Err(f) => fail!("Invalid options\n{}", f.to_err_msg()),
    };

    let opts = Options {
        showInvisibles: matches.opt_present("all"),
        reverse: matches.opt_present("reverse"),
        sortField: matches.opt_str("sort").map(|word| SortField::from_word(word)).unwrap_or(Name),
    };

    let strs = if matches.free.is_empty() {
        vec!("./".to_strbuf())
    }
    else {
        matches.free.clone()
    };

    for dir in strs.move_iter() {
        list(opts, Path::new(dir))
    }
}

fn list(options: Options, path: Path) {
    let paths = match fs::readdir(&path) {
        Ok(paths) => paths,
        Err(e) => fail!("readdir: {}", e),
    };

    let mut files = paths.iter().map(|path| File::from_path(path)).collect();
    options.sort(&mut files);
    if options.reverse {
        files.reverse();
    }

    let columns = defaultColumns();
    let num_columns = columns.len();

    let table: Vec<Vec<StrBuf>> = files.iter()
        .filter(|&f| options.show(f))
        .map(|f| columns.iter().map(|c| f.display(c)).collect())
        .collect();

    let lengths: Vec<Vec<uint>> = table.iter()
        .map(|row| row.iter().map( |col| colours::strip_formatting(col).len() ).collect())
        .collect();

    let maxes: Vec<uint> = range(0, num_columns)
        .map(|n| lengths.iter().map(|row| *row.get(n)).max().unwrap())
        .collect();

    for (field_lengths, row) in lengths.iter().zip(table.iter()) {
        let mut first = true;
        for ((column_length, cell), field_length) in maxes.iter().zip(row.iter()).zip(field_lengths.iter()) {
            if first {
                first = false;
            } else {
                print!(" ");
            }
            print!("{}", cell.as_slice());
            for _ in range(*field_length, *column_length) {
                print!(" ");
            }
        }
        print!("\n");
    }
}
