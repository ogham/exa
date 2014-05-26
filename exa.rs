#![feature(phase)]
extern crate regex;
#[phase(syntax)] extern crate regex_macros;

use std::os;
use std::io::fs;

use file::File;
use options::Options;

pub mod colours;
pub mod column;
pub mod format;
pub mod file;
pub mod unix;
pub mod options;

fn main() {
    let args = os::args().iter()
        .map(|x| x.to_strbuf())
        .collect();
    
    match Options::getopts(args) {
        Err(err) => println!("Invalid options:\n{}", err.to_err_msg()),
        Ok(opts) => {
            let strs = if opts.dirs.is_empty() {
                vec!("./".to_strbuf())
            }
            else {
                opts.dirs.clone()
            };
            
            for dir in strs.move_iter() {
                exa(&opts, Path::new(dir))
            }
        }
    };
}

fn exa(options: &Options, path: Path) {
    let paths = match fs::readdir(&path) {
        Ok(paths) => paths,
        Err(e) => fail!("readdir: {}", e),
    };

    let files: Vec<File> = options.transform_files(paths.iter().map(|path| File::from_path(path)).collect());
    let columns = options.columns();

    let table: Vec<Vec<String>> = files.iter()
        .filter(|&f| options.show(f))
        .map(|f| columns.iter().map(|c| f.display(c)).collect())
        .collect();

    let lengths: Vec<Vec<uint>> = table.iter()
        .map(|row| row.iter().map(|col| colours::strip_formatting(col).len()).collect())
        .collect();

    let maxes: Vec<uint> = range(0, columns.len())
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
