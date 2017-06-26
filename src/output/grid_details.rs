use std::io::{Write, Result as IOResult};
use std::sync::Arc;

use ansi_term::ANSIStrings;
use users::UsersCache;
use term_grid as grid;

use fs::{Dir, File};
use fs::feature::xattr::FileAttributes;

use output::cell::TextCell;
use output::column::Column;
use output::colours::Colours;
use output::details::{Table, Environment, Options as DetailsOptions};
use output::grid::Options as GridOptions;
use output::file_name::{Classify, LinkStyle};


pub struct Render<'a> {
    pub dir: Option<&'a Dir>,
    pub files: Vec<File<'a>>,
    pub colours: &'a Colours,
    pub classify: Classify,
    pub grid: &'a GridOptions,
    pub details: &'a DetailsOptions,
}

impl<'a> Render<'a> {
    pub fn render<W: Write>(&self, w: &mut W) -> IOResult<()> {

        let columns_for_dir = match self.details.columns {
            Some(cols) => cols.for_dir(self.dir),
            None => Vec::new(),
        };

        let env = Arc::new(Environment::default());

        let (cells, file_names) = {

            let first_table = self.make_table(env.clone(), &*columns_for_dir, self.colours, self.classify);

            let cells = self.files.iter()
                              .map(|file| first_table.cells_for_file(file, file_has_xattrs(file)))
                              .collect::<Vec<_>>();

            let file_names = self.files.iter()
                                  .map(|file| first_table.filename(file, LinkStyle::JustFilenames).promote())
                                  .collect::<Vec<_>>();

            (cells, file_names)
        };

        let mut last_working_table = self.make_grid(env.clone(), 1, &columns_for_dir, &file_names, cells.clone(), self.colours, self.classify);

        for column_count in 2.. {
            let grid = self.make_grid(env.clone(), column_count, &columns_for_dir, &file_names, cells.clone(), self.colours, self.classify);

            let the_grid_fits = {
                let d = grid.fit_into_columns(column_count);
                d.is_complete() && d.width() <= self.grid.console_width
            };

            if the_grid_fits {
                last_working_table = grid;
            }
            else {
                return write!(w, "{}", last_working_table.fit_into_columns(column_count - 1));
            }
        }

        Ok(())
    }

    fn make_table<'g>(&'g self, env: Arc<Environment<UsersCache>>, columns_for_dir: &'g [Column], colours: &'g Colours, classify: Classify) -> Table<UsersCache> {
        let mut table = Table {
            columns: columns_for_dir,
            colours, classify, env,
            xattr: self.details.xattr,
            rows: Vec::new(),
        };

        if self.details.header { table.add_header() }
        table
    }

    fn make_grid<'g>(&'g self, env: Arc<Environment<UsersCache>>, column_count: usize, columns_for_dir: &'g [Column], file_names: &[TextCell], cells: Vec<Vec<TextCell>>, colours: &'g Colours, classify: Classify) -> grid::Grid {
        let mut tables = Vec::new();
        for _ in 0 .. column_count {
            tables.push(self.make_table(env.clone(), columns_for_dir, colours, classify));
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
                for column in &columns {
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
            for column in &columns {
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


fn file_has_xattrs(file: &File) -> bool {
    match file.path.attributes() {
        Ok(attrs) => !attrs.is_empty(),
        Err(_) => false,
    }
}
