use std::iter::repeat;

use term_grid as grid;

use column::Cell;
use dir::Dir;
use file::File;
use output::details::{Details, Table};
use output::grid::Grid;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct GridDetails {
    pub grid: Grid,
    pub details: Details,
}

impl GridDetails {
    pub fn view(&self, dir: Option<&Dir>, files: &[File]) {
        let mut last_working_table = self.make_grid(1, dir, files);

        for column_count in 2.. {
            let grid = self.make_grid(column_count, dir, files);

            if grid.fit_into_columns(column_count).width() <= self.grid.console_width {
                last_working_table = grid;
            }
            else {
                print!("{}", last_working_table.fit_into_columns(column_count - 1));
                return;
            }
        }
    }

    pub fn make_grid(&self, column_count: usize, dir: Option<&Dir>, files: &[File]) -> grid::Grid {
        let make_table = || {
            let mut table = Table::with_options(self.details.colours, self.details.columns.for_dir(dir));
            if self.details.header { table.add_header() }
            table
        };

        let mut tables: Vec<_> = repeat(()).map(|_| make_table()).take(column_count).collect();

        let mut height = files.len() / column_count;
        if files.len() % column_count != 0 {
            height += 1;
        }

        for (i, file) in files.iter().enumerate() {
            tables[i / height].add_file(file, 0, false, false);
        }

        let columns: Vec<_> = tables.iter().map(|t| t.print_table(false, false)).collect();

        let direction = grid::Direction::TopToBottom;
        let mut grid = grid::Grid::new(grid::GridOptions {
            direction:        direction,
            separator_width:  4,
        });

        for column in columns.iter() {
            for cell in column.iter() {
                let cell = grid::Cell {
                    contents: cell.text.clone(),
                    width:    cell.length,
                };

                grid.add(cell);
            }
        }

        grid
    }
}
