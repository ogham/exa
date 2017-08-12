//! The grid-details view lists several details views side-by-side.

use std::io::{Write, Result as IOResult};

use ansi_term::ANSIStrings;
use term_grid as grid;

use fs::{Dir, File};
use fs::feature::xattr::FileAttributes;
use fs::filter::FileFilter;

use output::cell::TextCell;
use output::colours::Colours;
use output::details::{Options as DetailsOptions, Row as DetailsRow, Render as DetailsRender};
use output::grid::Options as GridOptions;
use output::file_name::FileStyle;
use output::table::{Table, Row as TableRow, Options as TableOptions};
use output::tree::{TreeParams, TreeDepth};


pub struct Render<'a> {

    /// The directory that’s being rendered here.
    /// We need this to know which columns to put in the output.
    pub dir: Option<&'a Dir>,

    /// The files that have been read from the directory. They should all
    /// hold a reference to it.
    pub files: Vec<File<'a>>,

    /// How to colour various pieces of text.
    pub colours: &'a Colours,

    /// How to format filenames.
    pub style: &'a FileStyle,

    /// The grid part of the grid-details view.
    pub grid: &'a GridOptions,

    /// The details part of the grid-details view.
    pub details: &'a DetailsOptions,

    /// How to filter files after listing a directory. The files in this
    /// render will already have been filtered and sorted, but any directories
    /// that we recurse into will have to have this applied.
    pub filter: &'a FileFilter,
}

impl<'a> Render<'a> {
    pub fn details(&self) -> DetailsRender<'a> {
        DetailsRender {
            dir: self.dir.clone(),
            files: Vec::new(),
            colours: self.colours,
            style: self.style,
            opts: self.details,
            recurse: None,
            filter: self.filter,
        }
    }

    pub fn render<W: Write>(&self, w: &mut W) -> IOResult<()> {

        let options = self.details.table.as_ref().expect("Details table options not given!");

        let drender = self.details();

        let (first_table, _) = self.make_table(options, &drender);

        let rows = self.files.iter()
                       .map(|file| first_table.row_for_file(file, file_has_xattrs(file)))
                       .collect::<Vec<TableRow>>();

        let file_names = self.files.iter()
                             .map(|file| self.style.for_file(file, self.colours).paint().promote())
                             .collect::<Vec<TextCell>>();

        let mut last_working_table = self.make_grid(1, options, &file_names, rows.clone(), &drender);

        for column_count in 2.. {
            let grid = self.make_grid(column_count, options, &file_names, rows.clone(), &drender);

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

    fn make_table<'t>(&'a self, options: &'a TableOptions, drender: &DetailsRender) -> (Table<'a>, Vec<DetailsRow>) {
        let mut table = Table::new(options, self.dir, self.colours);
        let mut rows = Vec::new();

        if self.details.header {
            let row = table.header_row();
            table.add_widths(&row);
            rows.push(drender.render_header(row));
        }

        (table, rows)
    }

    fn make_grid(&'a self, column_count: usize, options: &'a TableOptions, file_names: &[TextCell], rows: Vec<TableRow>, drender: &DetailsRender) -> grid::Grid {

        let mut tables = Vec::new();
        for _ in 0 .. column_count {
            tables.push(self.make_table(options, drender));
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
            let details_row = drender.render_file(row, file_name.clone(), TreeParams::new(TreeDepth::root(), false));
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
