#![feature(phase)]
extern crate regex;
#[phase(plugin)] extern crate regex_macros;
extern crate ansi_term;
extern crate unicode;

use std::os;

use file::File;
use dir::Dir;
use column::{Column, Left};
use options::{Options, Details, Lines, Grid};
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
pub mod term;

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
    
    for dir_name in opts.dirs.clone().into_iter() {
        if first {
            first = false;
        }
        else {
            print!("\n");
        }

        match Dir::readdir(Path::new(dir_name.clone())) {
            Ok(dir) => {
                let unsorted_files = dir.files();
                let files: Vec<&File> = opts.transform_files(&unsorted_files);

                if print_dir_names {
                    println!("{}:", dir_name);
                }

                match opts.view {
                    Details(ref cols) => details_view(opts, cols, files),
                    Lines => lines_view(files),
                    Grid(across, width) => grid_view(across, width, files),
                }
            }
            Err(e) => {
                println!("{}: {}", dir_name, e);
                return;
            }
        };
    }
}

fn lines_view(files: Vec<&File>) {
    for file in files.iter() {
        println!("{}", file.file_name());
    }
}

fn grid_view(across: bool, console_width: uint, files: Vec<&File>) {
    let max_column_length = files.iter().map(|f| f.file_name_width()).max().unwrap();
    let num_columns = (console_width + 1) / (max_column_length + 1);
    let count = files.len();

    let mut num_rows = count / num_columns;
    if count % num_columns != 0 {
        num_rows += 1;
    }
    
    for y in range(0, num_rows) {
        for x in range(0, num_columns) {
            let num = if across {
                y * num_columns + x
            }
            else {
                y + num_rows * x
            };
            
            if num >= count {
                continue;
            }
            
            let file = files[num];
            let file_name = file.name.clone();
            let styled_name = file.file_colour().paint(file_name.as_slice());
            if x == num_columns - 1 {
                print!("{}", styled_name);
            }
            else {
                print!("{}", Left.pad_string(&styled_name, max_column_length - file_name.len() + 1));
            }
        }
        print!("\n");
    }
}

fn details_view(options: &Options, columns: &Vec<Column>, files: Vec<&File>) {
    // The output gets formatted into columns, which looks nicer. To
    // do this, we have to write the results into a table, instead of
    // displaying each file immediately, then calculating the maximum
    // width of each column based on the length of the results and
    // padding the fields during output.

    let mut cache = Unix::empty_cache();

    let mut table: Vec<Vec<String>> = files.iter()
        .map(|f| columns.iter().map(|c| f.display(c, &mut cache)).collect())
        .collect();

    if options.header {
        table.insert(0, columns.iter().map(|c| Plain.underline().paint(c.header())).collect());
    }

    // Each column needs to have its invisible colour-formatting
    // characters stripped before it has its width calculated, or the
    // width will be incorrect and the columns won't line up properly.
    // This is fairly expensive to do (it uses a regex), so the
    // results are cached.

    let lengths: Vec<Vec<uint>> = table.iter()
        .map(|row| row.iter().map(|col| strip_formatting(col.clone()).len()).collect())
        .collect();

    let column_widths: Vec<uint> = range(0, columns.len())
        .map(|n| lengths.iter().map(|row| row[n]).max().unwrap())
        .collect();

    for (field_widths, row) in lengths.iter().zip(table.iter()) {
        for (num, column) in columns.iter().enumerate() {
            if num != 0 {
                print!(" ");
            }

            if num == columns.len() - 1 {
                print!("{}", row.get(num));
            }
            else {
                let padding = column_widths[num] - field_widths[num];
                print!("{}", column.alignment().pad_string(row.get(num), padding));
            }
        }
        print!("\n");
    }
}
