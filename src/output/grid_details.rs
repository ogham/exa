use std::sync::Arc;

use ansi_term::ANSIStrings;
use users::UsersCache;
use term_grid as grid;

use fs::{Dir, File};
use fs::feature::xattr::FileAttributes;

use output::cell::TextCell;
use output::column::Column;
use output::details::{Details, Table, Environment};
use output::grid::Grid;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct GridDetails {
    pub grid: Grid,
    pub details: Details,
}

fn file_has_xattrs(file: &File) -> bool {
    match file.path.attributes() {
        Ok(attrs) => !attrs.is_empty(),
        Err(_) => false,
    }
}

impl GridDetails {
    pub fn view(&self, dir: Option<&Dir>, files: Vec<File>) {
        let columns_for_dir = match self.details.columns {
            Some(cols) => cols.for_dir(dir),
            None => Vec::new(),
        };

        let env = Arc::new(Environment::default());

        let (cells, file_names) = {

            let first_table = self.make_table(env.clone(), &*columns_for_dir);

            let cells = files.iter()
                              .map(|file| first_table.cells_for_file(file, file_has_xattrs(file)))
                              .collect::<Vec<_>>();

            let file_names = files.into_iter()
                                  .map(|file| first_table.filename_cell(file, false))
                                  .collect::<Vec<_>>();

            (cells, file_names)
        };

        let mut last_working_table = self.make_grid(env.clone(), 1, &columns_for_dir, &file_names, cells.clone());

        for column_count in 2.. {
            let grid = self.make_grid(env.clone(), column_count, &columns_for_dir, &file_names, cells.clone());

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

    fn make_table<'a>(&'a self, env: Arc<Environment<UsersCache>>, columns_for_dir: &'a [Column]) -> Table<UsersCache> {
        let mut table = Table {
            columns: columns_for_dir,
            opts: &self.details,
            env: env,

            rows: Vec::new(),
        };

        if self.details.header { table.add_header() }
        table
    }

    fn make_grid<'a>(&'a self, env: Arc<Environment<UsersCache>>, column_count: usize, columns_for_dir: &'a [Column], file_names: &[TextCell], cells: Vec<Vec<TextCell>>) -> grid::Grid {
        let mut tables = Vec::new();
        for _ in 0 .. column_count {
            tables.push(self.make_table(env.clone(), columns_for_dir));
        }

        let mut num_cells = cells.len();
        if self.details.header {
            num_cells += column_count;
        }

        let original_height = divide_rounding_up(cells.len(), column_count);
        let height = divide_rounding_up(num_cells, column_count);

        for (i, (file_name, row)) in file_names.iter().zip(cells.into_iter()).enumerate() {
            let index = if self.grid.across {
                    i % column_count
                }
                else {
                    i / original_height
                };

            tables[index].add_file_with_cells(row, file_name.clone(), 0, false);
        }

        let columns: Vec<_> = tables.into_iter().map(|t| t.print_table()).collect();

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
                            contents: ANSIStrings(&column[row].contents).to_string(),
                            width:    *column[row].width,
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
                        contents: ANSIStrings(&cell.contents).to_string(),
                        width:    *cell.width,
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