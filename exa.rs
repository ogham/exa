#![feature(phase)]
extern crate regex;
#[phase(plugin)] extern crate regex_macros;

use std::os;

use file::File;
use dir::Dir;
use options::Options;

pub mod colours;
pub mod column;
pub mod dir;
pub mod format;
pub mod file;
pub mod filetype;
pub mod unix;
pub mod options;
pub mod sort;

fn main() {
    let args = os::args();
    
    match Options::getopts(args) {
        Err(err) => println!("Invalid options:\n{}", err),
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
                exa(&opts, dir)
            }
        }
    };
}

fn exa(options: &Options, string: String) {
    let path = Path::new(string.clone());

    let dir = match Dir::readdir(path) {
        Ok(dir) => dir,
        Err(e) => {
            println!("{}: {}", string, e);
            return;
        }
    };
    
    let unsorted_files = dir.files();
    let files: Vec<&File> = options.transform_files(&unsorted_files);

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
        for (((column_length, cell), field_length), (num, column)) in column_widths.iter().zip(row.iter()).zip(field_lengths.iter()).zip(options.columns.iter().enumerate()) {  // this is getting messy
            if num != 0 {
                print!(" ");
            }
            
            if num == options.columns.len() - 1 {
                print!("{}", cell);
            }
            else {
                print!("{}", column.alignment().pad_string(cell, *field_length, *column_length));
            }
        }
        print!("\n");
    }
}
