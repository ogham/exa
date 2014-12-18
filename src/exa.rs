#![feature(phase, globs)]
extern crate regex;
#[phase(plugin)] extern crate regex_macros;
extern crate ansi_term;
extern crate number_prefix;
extern crate unicode;
extern crate users;

use std::io::FileType;
use std::io::fs;
use std::iter::AdditiveIterator;
use std::os;
use std::cmp::max;

use column::Alignment::Left;
use column::Column;
use dir::Dir;
use file::File;
use options::{Options, View};

use ansi_term::Style::Plain;
use ansi_term::strip_formatting;

use users::OSUsers;

pub mod column;
pub mod dir;
pub mod file;
pub mod filetype;
pub mod options;
pub mod term;

fn main() {
    let args: Vec<String> = os::args();

    match Options::getopts(args) {
        Err(error_code) => os::set_exit_status(error_code),
        Ok(options) => exa(&options),
    };
}

fn exa(opts: &Options) {
    let mut dirs: Vec<String> = vec![];
    let mut files: Vec<File> = vec![];

    // Separate the user-supplied paths into directories and files.
    // Files are shown first, and then each directory is expanded
    // and listed second.
    for file in opts.path_strs.iter() {
        let path = Path::new(file);
        match fs::stat(&path) {
            Ok(stat) => {
                if !opts.list_dirs && stat.kind == FileType::Directory {
                    dirs.push(file.clone());
                }
                else {
                    // May as well reuse the stat result from earlier
                    // instead of just using File::from_path().
                    files.push(File::with_stat(stat, &path, None));
                }
            }
            Err(e) => println!("{}: {}", file, e),
        }
    }

    // It's only worth printing out directory names if the user supplied
    // more than one of them.
    let print_dir_names = opts.path_strs.len() > 1;
    let mut first = files.is_empty();

    if !files.is_empty() {
        view(opts, files);
    }

    for dir_name in dirs.iter() {
        if first {
            first = false;
        }
        else {
            print!("\n");
        }

        match Dir::readdir(Path::new(dir_name.clone())) {
            Ok(dir) => {
                let unsorted_files = dir.files();
                let files: Vec<File> = opts.transform_files(unsorted_files);

                if print_dir_names {
                    println!("{}:", dir_name);
                }

                view(opts, files);
            }
            Err(e) => {
                println!("{}: {}", dir_name, e);
                return;
            }
        };
    }
}

fn view(options: &Options, files: Vec<File>) {
    match options.view {
        View::Details(ref cols) => details_view(options, cols, files),
        View::Lines => lines_view(files),
        View::Grid(across, width) => grid_view(across, width, files),
    }
}

fn lines_view(files: Vec<File>) {
    for file in files.iter() {
        println!("{}", file.file_name());
    }
}

fn fit_into_grid(across: bool, console_width: uint, files: &Vec<File>) -> Option<(uint, Vec<uint>)> {
    // TODO: this function could almost certainly be optimised...
    // surely not *all* of the numbers of lines are worth searching through!
    
    // Instead of numbers of columns, try to find the fewest number of *lines*
    // that the output will fit in.
    for num_lines in range(1, files.len()) {
    
        // The number of columns is the number of files divided by the number
        // of lines, *rounded up*.
        let mut num_columns = files.len() / num_lines;
        if files.len() % num_lines != 0 {
            num_columns += 1;
        }

        // Early abort: if there are so many columns that the width of the
        // *column separators* is bigger than the width of the screen, then
        // don't even try to tabulate it.
        // This is actually a necessary check, because the width is stored as
        // a uint, and making it go negative makes it huge instead, but it
        // also serves as a speed-up.
        let separator_width = (num_columns - 1) * 2;
        if console_width < separator_width {
            continue;
        }

        // Remove the separator width from the available space.
        let adjusted_width = console_width - separator_width;

        // Find the width of each column by adding the lengths of the file
        // names in that column up.
        let mut column_widths = Vec::from_fn(num_columns, |_| 0u);
        for (index, file) in files.iter().enumerate() {
            let index = if across {
                index % num_columns
            }
            else {
                index / num_lines
            };
            column_widths[index] = max(column_widths[index], file.name.len());
        }

        // If they all fit in the terminal, combined, then success!
        if column_widths.iter().map(|&x| x).sum() < adjusted_width {
            return Some((num_lines, column_widths));
        }
    }

    // If you get here you have really long file names.
    return None;
}

fn grid_view(across: bool, console_width: uint, files: Vec<File>) {
    if let Some((num_lines, widths)) = fit_into_grid(across, console_width, &files) {
        for y in range(0, num_lines) {
            for x in range(0, widths.len()) {
                let num = if across {
                    y * widths.len() + x
                }
                else {
                    y + num_lines * x
                };

                // Show whitespace in the place of trailing files
                if num >= files.len() {
                    continue;
                }

                let ref file = files[num];
                let styled_name = file.file_colour().paint(file.name.as_slice()).to_string();
                if x == widths.len() - 1 {
                    // The final column doesn't need to have trailing spaces
                    print!("{}", styled_name);
                }
                else {
                    assert!(widths[x] >= file.name.len());
                    print!("{}", Left.pad_string(&styled_name, widths[x] - file.name.len() + 2));
                }
            }
            print!("\n");
        }
    }
    else {
        // Drop down to lines view if the file names are too big for a grid
        lines_view(files);
    }
}

fn details_view(options: &Options, columns: &Vec<Column>, files: Vec<File>) {
    // The output gets formatted into columns, which looks nicer. To
    // do this, we have to write the results into a table, instead of
    // displaying each file immediately, then calculating the maximum
    // width of each column based on the length of the results and
    // padding the fields during output.

    let mut cache = OSUsers::empty_cache();

    let mut table: Vec<Vec<String>> = files.iter()
        .map(|f| columns.iter().map(|c| f.display(c, &mut cache)).collect())
        .collect();

    if options.header {
        table.insert(0, columns.iter().map(|c| Plain.underline().paint(c.header()).to_string()).collect());
    }

    // Each column needs to have its invisible colour-formatting
    // characters stripped before it has its width calculated, or the
    // width will be incorrect and the columns won't line up properly.
    // This is fairly expensive to do (it uses a regex), so the
    // results are cached.

    let lengths: Vec<Vec<uint>> = table.iter()
        .map(|row| row.iter().map(|col| strip_formatting(col.as_slice()).len()).collect())
        .collect();

    let column_widths: Vec<uint> = range(0, columns.len())
        .map(|n| lengths.iter().map(|row| row[n]).max().unwrap_or(0))
        .collect();

    for (field_widths, row) in lengths.iter().zip(table.iter()) {
        for (num, column) in columns.iter().enumerate() {
            if num != 0 {
                print!(" ");  // Separator
            }

            if num == columns.len() - 1 {
                // The final column doesn't need to have trailing spaces
                print!("{}", row[num]);
            }
            else {
                let padding = column_widths[num] - field_widths[num];
                print!("{}", column.alignment().pad_string(&row[num], padding));
            }
        }
        print!("\n");
    }
}
