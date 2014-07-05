#![feature(phase)]
extern crate regex;
#[phase(plugin)] extern crate regex_macros;
extern crate ansi_term;

use std::os;

use file::File;
use dir::Dir;
use options::Options;
use unix::Unix;

use ansi_term::{Paint, Plain, strip_formatting};

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
        Ok(opts) => exa(&opts),
    };
}

fn exa(opts: &Options) {
    let mut first = true;
    
    // It's only worth printing out directory names if the user supplied
    // more than one of them.
    let print_dir_names = opts.dirs.len() > 1;
    
    for dir_name in opts.dirs.clone().move_iter() {
        if first {
            first = false;
        }
        else {
            print!("\n");
        }

        match Dir::readdir(Path::new(dir_name.clone())) {
            Ok(dir) => {
                if print_dir_names { println!("{}:", dir_name); }
                lines_view(opts, dir);
            }
            Err(e) => {
                println!("{}: {}", dir_name, e);
                return;
            }
        };
    }
}

fn lines_view(options: &Options, dir: Dir) {
    let unsorted_files = dir.files();
    let files: Vec<&File> = options.transform_files(&unsorted_files);

    // The output gets formatted into columns, which looks nicer. To
    // do this, we have to write the results into a table, instead of
    // displaying each file immediately, then calculating the maximum
    // width of each column based on the length of the results and
    // padding the fields during output.

    let mut cache = Unix::empty_cache();

    let mut table: Vec<Vec<String>> = files.iter()
        .map(|f| options.columns.iter().map(|c| f.display(c, &mut cache)).collect())
        .collect();

    if options.header {
        table.unshift(options.columns.iter().map(|c| Plain.underline().paint(c.header())).collect());
    }

    // Each column needs to have its invisible colour-formatting
    // characters stripped before it has its width calculated, or the
    // width will be incorrect and the columns won't line up properly.
    // This is fairly expensive to do (it uses a regex), so the
    // results are cached.

    let lengths: Vec<Vec<uint>> = table.iter()
        .map(|row| row.iter().map(|col| strip_formatting(col.clone()).len()).collect())
        .collect();

    let column_widths: Vec<uint> = range(0, options.columns.len())
        .map(|n| lengths.iter().map(|row| *row.get(n)).max().unwrap())
        .collect();

    for (field_widths, row) in lengths.iter().zip(table.iter()) {
        for (num, column) in options.columns.iter().enumerate() {
            if num != 0 {
                print!(" ");
            }

            if num == options.columns.len() - 1 {
                print!("{}", row.get(num));
            }
            else {
                let padding = *column_widths.get(num) - *field_widths.get(num);
                print!("{}", column.alignment().pad_string(row.get(num), padding));
            }
        }
        print!("\n");
    }
}
