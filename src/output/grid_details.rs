use std::io::{Write, Result as IOResult};

use ansi_term::ANSIStrings;
use term_grid as grid;

use fs::{Dir, File};
use fs::feature::xattr::FileAttributes;

use options::FileFilter;
use output::cell::TextCell;
use output::column::Column;
use output::colours::Colours;
use output::details::{Options as DetailsOptions, Row as DetailsRow, Render as DetailsRender};
use output::grid::Options as GridOptions;
use output::file_name::{FileName, LinkStyle, Classify};
use output::table::{Table, Environment, Row as TableRow};
use output::tree::{TreeParams, TreeDepth};


pub struct Render<'a> {
    pub dir: Option<&'a Dir>,
    pub files: Vec<File<'a>>,
    pub colours: &'a Colours,
    pub classify: Classify,
    pub grid: &'a GridOptions,
    pub details: &'a DetailsOptions,
    pub filter: &'a FileFilter,
}

impl<'a> Render<'a> {
    pub fn details(&self) -> DetailsRender<'a> {
        DetailsRender {
            dir: self.dir.clone(),
            files: Vec::new(),
            colours: self.colours,
            classify: self.classify,
            opts: self.details,
            recurse: None,
            filter: self.filter,
        }
    }

    pub fn render<W: Write>(&self, w: &mut W) -> IOResult<()> {

        let columns_for_dir = match self.details.columns {
            Some(cols) => cols.for_dir(self.dir),
            None => Vec::new(),
        };

        let env = Environment::default();

        let drender = self.clone().details();

        let (first_table, _) = self.make_table(&env, &columns_for_dir, &drender);

        let rows = self.files.iter()
                       .map(|file| first_table.row_for_file(file, file_has_xattrs(file)))
                       .collect::<Vec<TableRow>>();

        let file_names = self.files.iter()
                             .map(|file| FileName::new(file, LinkStyle::JustFilenames, self.classify, self.colours).paint().promote())
                             .collect::<Vec<TextCell>>();

        let mut last_working_table = self.make_grid(&env, 1, &columns_for_dir, &file_names, rows.clone(), &drender);

        for column_count in 2.. {
            let grid = self.make_grid(&env, column_count, &columns_for_dir, &file_names, rows.clone(), &drender);

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

    fn make_table<'t>(&'a self, env: &'a Environment, columns_for_dir: &'a [Column], drender: &DetailsRender) -> (Table<'a>, Vec<DetailsRow>) {
        let mut table = Table::new(columns_for_dir, self.colours, env);
        let mut rows = Vec::new();

        if self.details.header {
            let row = table.header_row();
            table.add_widths(&row);
            rows.push(drender.render_header(row));
        }

        (table, rows)
    }

    fn make_grid(&'a self, env: &'a Environment, column_count: usize, columns_for_dir: &'a [Column], file_names: &[TextCell], rows: Vec<TableRow>, drender: &DetailsRender) -> grid::Grid {

        let mut tables = Vec::new();
        for _ in 0 .. column_count {
            tables.push(self.make_table(env.clone(), columns_for_dir, drender));
        }

        let mut num_cells = rows.len();
        if self.details.header {
            num_cells += column_count;
        }

        let original_height = divide_rounding_up(rows.len(), column_count);
        let height = divide_rounding_up(num_cells, column_count);

        for (i, (file_name, row)) in file_names.iter().zip(rows.into_iter()).enumerate() {
            let index = if self.grid.across {
                    i % column_count
                }
                else {
                    i / original_height
                };

            let (ref mut table, ref mut rows) = tables[index];
            table.add_widths(&row);
            let details_row = drender.render_file(row, file_name.clone(), TreeParams::new(TreeDepth(0), false));
            rows.push(details_row);
        }

        let columns: Vec<_> = tables.into_iter().map(|(table, details_rows)| {
            drender.iterate_with_table(table, details_rows).collect::<Vec<_>>()
        }).collect();

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
