use std::iter::repeat;

use term_grid as grid;

use column::{Column, Cell};
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
        let columns_for_dir = self.details.columns.for_dir(dir);
        let mut first_table = Table::with_options(self.details.colours, columns_for_dir.clone());
        let cells: Vec<_> = files.iter().map(|file| first_table.cells_for_file(file)).collect();

        let mut last_working_table = self.make_grid(1, &*columns_for_dir, files, cells.clone());

        for column_count in 2.. {
            let grid = self.make_grid(column_count, &*columns_for_dir, files, cells.clone());

            let the_grid_fits = {
                let d = grid.fit_into_columns(column_count);
                d.is_complete() && d.width() <= self.grid.console_width
            };

            if the_grid_fits {
                last_working_table = grid;
            }
            else {
                print!("{}", last_working_table.fit_into_columns(column_count - 1));
                return;
            }
        }
    }

    pub fn make_grid(&self, column_count: usize, columns_for_dir: &[Column], files: &[File], cells: Vec<Vec<Cell>>) -> grid::Grid {
        let make_table = || {
            let mut table = Table::with_options(self.details.colours, columns_for_dir.into());
            if self.details.header { table.add_header() }
            table
        };

        let mut tables: Vec<_> = repeat(()).map(|_| make_table()).take(column_count).collect();

        let mut height = cells.len() / column_count;
        if cells.len() % column_count != 0 {
            height += 1;
        }

        for (i, (file, row)) in files.iter().zip(cells.into_iter()).enumerate() {
            tables[i / height].add_file_with_cells(row, file, 0, false, false);
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
