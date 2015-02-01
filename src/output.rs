use std::cmp::max;
use std::iter::{AdditiveIterator, repeat};

use column::{Column, Cell};
use column::Alignment::Left;
use dir::Dir;
use file::File;
use options::Columns;
use users::OSUsers;

use ansi_term::Style::Plain;

#[derive(PartialEq, Copy, Debug)]
pub enum View {
    Details(Columns, bool),
    Lines,
    Grid(bool, usize),
}

impl View {
    pub fn view(&self, dir: Option<&Dir>, files: &[File]) {
        match *self {
            View::Grid(across, width)       => grid_view(across, width, files),
            View::Details(ref cols, header) => details_view(&*cols.for_dir(dir), files, header),
            View::Lines                     => lines_view(files),
        }
    }
}

/// The lines view literally just displays each file, line-by-line.
fn lines_view(files: &[File]) {
    for file in files.iter() {
        println!("{}", file.file_name_view().text);
    }
}

fn fit_into_grid(across: bool, console_width: usize, files: &[File]) -> Option<(usize, Vec<usize>)> {
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
        // a usize, and making it go negative makes it huge instead, but it
        // also serves as a speed-up.
        let separator_width = (num_columns - 1) * 2;
        if console_width < separator_width {
            continue;
        }

        // Remove the separator width from the available space.
        let adjusted_width = console_width - separator_width;

        // Find the width of each column by adding the lengths of the file
        // names in that column up.
        let mut column_widths: Vec<usize> = repeat(0).take(num_columns).collect();
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

fn grid_view(across: bool, console_width: usize, files: &[File]) {
    if let Some((num_lines, widths)) = fit_into_grid(across, console_width, files) {
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

fn details_view(columns: &[Column], files: &[File], header: bool) {
    // The output gets formatted into columns, which looks nicer. To
    // do this, we have to write the results into a table, instead of
    // displaying each file immediately, then calculating the maximum
    // width of each column based on the length of the results and
    // padding the fields during output.

    let mut cache = OSUsers::empty_cache();

    let mut table: Vec<Vec<Cell>> = files.iter()
        .map(|f| columns.iter().map(|c| f.display(c, &mut cache)).collect())
        .collect();

    if header {
        table.insert(0, columns.iter().map(|c| Cell::paint(Plain.underline(), c.header())).collect());
    }

    let column_widths: Vec<usize> = range(0, columns.len())
        .map(|n| table.iter().map(|row| row[n].length).max().unwrap_or(0))
        .collect();

    for row in table.iter() {
        for (num, column) in columns.iter().enumerate() {
            if num != 0 {
                print!(" ");  // Separator
            }

            if num == columns.len() - 1 {
                // The final column doesn't need to have trailing spaces
                print!("{}", row[num].text);
            }
            else {
                let padding = column_widths[num] - row[num].length;
                print!("{}", column.alignment().pad_string(&row[num].text, padding));
            }
        }
        print!("\n");
    }
}
