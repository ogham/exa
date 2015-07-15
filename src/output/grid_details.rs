use std::iter::repeat;

use users::OSUsers;
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

    fn make_table(&self, columns_for_dir: &[Column]) -> Table<OSUsers> {
        let mut table = Table::with_options(self.details.colours, columns_for_dir.into());
        if self.details.header { table.add_header() }
        table
    }

    fn make_grid(&self, column_count: usize, columns_for_dir: &[Column], files: &[File], cells: Vec<Vec<Cell>>) -> grid::Grid {
        let mut tables: Vec<_> = repeat(()).map(|_| self.make_table(columns_for_dir)).take(column_count).collect();

        let mut num_cells = cells.len();
        if self.details.header {
            num_cells += column_count;
        }

        let original_height = divide_rounding_up(cells.len(), column_count);
        let height = divide_rounding_up(num_cells, column_count);

        for (i, (file, row)) in files.iter().zip(cells.into_iter()).enumerate() {
            let index = if self.grid.across {
                    i % column_count
                }
                else {
                    i / original_height
                };

            tables[index].add_file_with_cells(row, file, 0, false, false);
        }

        let columns: Vec<_> = tables.iter().map(|t| t.print_table(false, false)).collect();

        let direction = if self.grid.across { grid::Direction::LeftToRight }
                                       else { grid::Direction::TopToBottom };

        let mut grid = grid::Grid::new(grid::GridOptions {
            direction:  direction,
            filling:    grid::Filling::Spaces(4),
        });

        if self.grid.across {
            for row in 0 .. height {
                for column in columns.iter() {
                    if row < column.len() {
                        let cell = grid::Cell {
                            contents: column[row].text.clone(),
                            width:    column[row].length,
                        };

                        grid.add(cell);
                    }
                }
            }
        }
        else {
            for column in columns.iter() {
                for cell in column.iter() {
                    let cell = grid::Cell {
                        contents: cell.text.clone(),
                        width:    cell.length,
                    };

                    grid.add(cell);
                }
            }
        }

        grid
    }
}


fn divide_rounding_up(a: usize, b: usize) -> usize {
    let mut result = a / b;
    if a % b != 0 { result += 1; }
    result
}