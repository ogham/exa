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
pub mod sort;

fn main() {
    let args = os::args();
    
    match Options::getopts(args) {
        Err(err) => println!("Invalid options:\n{}", err.to_err_msg()),
        Ok(opts) => {

            // Default to listing the current directory when a target
            // isn't specified (mimic the behaviour of ls)
            let strs = if opts.dirs.is_empty() {
                vec!(".".to_string())
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

    let unordered_files: Vec<File> = paths.iter().map(|path| File::from_path(path)).collect();
    let files: Vec<&File> = options.transform_files(&unordered_files);

    // The output gets formatted into columns, which looks nicer. To
    // do this, we have to write the results into a table, instead of
    // displaying each file immediately, then calculating the maximum
    // width of each column based on the length of the results and
    // padding the fields during output.

    let table: Vec<Vec<String>> = files.iter()
        .map(|f| options.columns.iter().map(|c| f.display(c)).collect())
        .collect();

    // Each column needs to have its invisible colour-formatting
    // characters stripped before it has its width calculated, or the
    // width will be incorrect and the columns won't line up properly.
    // This is fairly expensive to do (it uses a regex), so the
    // results are cached.

    let lengths: Vec<Vec<uint>> = table.iter()
        .map(|row| row.iter().map(|col| colours::strip_formatting(col).len()).collect())
        .collect();

    let column_widths: Vec<uint> = range(0, options.columns.len())
        .map(|n| lengths.iter().map(|row| *row.get(n)).max().unwrap())
        .collect();

    for (field_lengths, row) in lengths.iter().zip(table.iter()) {
        let mut first = true;
        for (((column_length, cell), field_length), column) in column_widths.iter().zip(row.iter()).zip(field_lengths.iter()).zip(options.columns.iter()) {  // this is getting messy
            if first {
                first = false;
            } else {
                print!(" ");
            }
            print!("{}", column.alignment().pad_string(cell, *field_length, *column_length));
        }
        print!("\n");
    }
}
