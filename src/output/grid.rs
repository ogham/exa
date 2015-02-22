use column::Alignment::Left;
use file::File;
use super::lines::lines_view;

use std::cmp::max;
use std::iter::{AdditiveIterator, repeat};

#[derive(PartialEq, Debug, Copy)]
pub struct Grid {
    pub across: bool,
    pub console_width: usize,
}

impl Grid {
    fn fit_into_grid(&self, files: &[File]) -> Option<(usize, Vec<usize>)> {
        // TODO: this function could almost certainly be optimised...
        // surely not *all* of the numbers of lines are worth searching through!

        // Instead of numbers of columns, try to find the fewest number of *lines*
        // that the output will fit in.
        for num_lines in 1 .. files.len() {

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
            if self.console_width < separator_width {
                continue;
            }

            // Remove the separator width from the available space.
            let adjusted_width = self.console_width - separator_width;

            // Find the width of each column by adding the lengths of the file
            // names in that column up.
            let mut column_widths: Vec<usize> = repeat(0).take(num_columns).collect();
            for (index, file) in files.iter().enumerate() {
                let index = if self.across {
                    index % num_columns
                }
                else {
                    index / num_lines
                };
                column_widths[index] = max(column_widths[index], file.file_name_width());
            }

            // If they all fit in the terminal, combined, then success!
            if column_widths.iter().map(|&x| x).sum() < adjusted_width {
                return Some((num_lines, column_widths));
            }
        }

        // If you get here you have really long file names.
        return None;
    }

    pub fn view(&self, files: &[File]) {
        if let Some((num_lines, widths)) = self.fit_into_grid(files) {
            for y in 0 .. num_lines {
                for x in 0 .. widths.len() {
                    let num = if self.across {
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
                        assert!(widths[x] >= file.file_name_width());
                        print!("{}", Left.pad_string(&styled_name, widths[x] - file.file_name_width() + 2));
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
}
