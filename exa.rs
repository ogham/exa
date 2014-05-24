#![feature(phase)]
extern crate regex;
#[phase(syntax)] extern crate regex_macros;

extern crate getopts;
use std::os;
use std::io::fs;

use file::File;
use column::defaultColumns;

pub mod colours;
pub mod column;
pub mod format;
pub mod file;
pub mod unix;

struct Options {
    showInvisibles: bool,
}

fn main() {
    let args: Vec<StrBuf> = os::args().iter()
        .map(|x| x.to_strbuf())
        .collect();

    let opts = ~[
        getopts::optflag("a", "all", "show dot-files")
    ];

    let matches = match getopts::getopts(args.tail(), opts) {
        Ok(m) => m,
        Err(f) => fail!("Invalid options\n{}", f.to_err_msg()),
    };

    let opts = Options {
        showInvisibles: matches.opt_present("all")
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

fn list(opts: Options, path: Path) {
    let mut files = match fs::readdir(&path) {
        Ok(files) => files,
        Err(e) => fail!("readdir: {}", e),
    };
    files.sort_by(|a, b| a.filename_str().cmp(&b.filename_str()));

    let columns = defaultColumns();

    let table: Vec<Vec<StrBuf>> = files.iter()
        .map(|p| File::from_path(p))
        .filter(|f| !f.is_dotfile() || opts.showInvisibles )
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
