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
        getopts::optopt("s", "sort", "field to sort by", "WORD"),
    ];

    let matches = match getopts::getopts(args.tail(), opts) {
        Ok(m) => m,
        Err(f) => fail!("Invalid options\n{}", f.to_err_msg()),
    };

    let opts = Options {
        showInvisibles: matches.opt_present("all"),
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

    let columns = defaultColumns();
    let table: Vec<Vec<StrBuf>> = files.iter()
        .filter(|&f| options.show(f))
        .map(|f| columns.iter().map(|c| f.display(c)).collect())
        .collect();

    let maxes: Vec<uint> = range(0, columns.len())
        .map(|n| table.iter().map(|row| colours::strip_formatting(row.get(n)).len()).max().unwrap())
        .collect();

    for row in table.iter() {
        let mut first = true;
        for (length, cell) in maxes.iter().zip(row.iter()) {
            if first {
                first = false;
            } else {
                print!(" ");
            }
            print!("{}", cell.as_slice());
            for _ in range(cell.len(), *length) {
                print!(" ");
            }
        }
        print!("\n");
    }
}
