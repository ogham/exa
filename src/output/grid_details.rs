//! The grid-details view lists several details views side-by-side.

use std::io::{self, Write};

use ansi_term::ANSIStrings;
use term_grid as grid;

use crate::fs::{Dir, File};
use crate::fs::feature::git::GitCache;
use crate::fs::feature::xattr::FileAttributes;
use crate::fs::filter::FileFilter;
use crate::output::cell::TextCell;
use crate::output::details::{Options as DetailsOptions, Row as DetailsRow, Render as DetailsRender};
use crate::output::file_name::Options as FileStyle;
use crate::output::grid::Options as GridOptions;
use crate::output::table::{Table, Row as TableRow, Options as TableOptions};
use crate::output::tree::{TreeParams, TreeDepth};
use crate::theme::Theme;


#[derive(PartialEq, Debug)]
pub struct Options {
    pub grid: GridOptions,
    pub details: DetailsOptions,
    pub row_threshold: RowThreshold,
}

impl Options {
    pub fn to_details_options(&self) -> &DetailsOptions {
        &self.details
    }
}


/// The grid-details view can be configured to revert to just a details view
/// (with one column) if it wouldn’t produce enough rows of output.
///
/// Doing this makes the resulting output look a bit better: when listing a
/// small directory of four files in four columns, the files just look spaced
/// out and it’s harder to see what’s going on. So it can be enabled just for
/// larger directory listings.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum RowThreshold {

    /// Only use grid-details view if it would result in at least this many
    /// rows of output.
    MinimumRows(usize),

    /// Use the grid-details view no matter what.
    AlwaysGrid,
}


pub struct Render<'a> {

    /// The directory that’s being rendered here.
    /// We need this to know which columns to put in the output.
    pub dir: Option<&'a Dir>,

    /// The files that have been read from the directory. They should all
    /// hold a reference to it.
    pub files: Vec<File<'a>>,

    /// How to colour various pieces of text.
    pub theme: &'a Theme,

    /// How to format filenames.
    pub file_style: &'a FileStyle,

    /// The grid part of the grid-details view.
    pub grid: &'a GridOptions,

    /// The details part of the grid-details view.
    pub details: &'a DetailsOptions,

    /// How to filter files after listing a directory. The files in this
    /// render will already have been filtered and sorted, but any directories
    /// that we recurse into will have to have this applied.
    pub filter: &'a FileFilter,

    /// The minimum number of rows that there need to be before grid-details
    /// mode is activated.
    pub row_threshold: RowThreshold,

    /// Whether we are skipping Git-ignored files.
    pub git_ignoring: bool,

    pub git: Option<&'a GitCache>,

    pub console_width: usize,
}

impl<'a> Render<'a> {

    /// Create a temporary Details render that gets used for the columns of
    /// the grid-details render that’s being generated.
    ///
    /// This includes an empty files vector because the files get added to
    /// the table in *this* file, not in details: we only want to insert every
    /// *n* files into each column’s table, not all of them.
    fn details_for_column(&self) -> DetailsRender<'a> {
        DetailsRender {
            dir:           self.dir,
            files:         Vec::new(),
            theme:         self.theme,
            file_style:    self.file_style,
            opts:          self.details,
            recurse:       None,
            filter:        self.filter,
            git_ignoring:  self.git_ignoring,
            git:           self.git,
        }
    }

    /// Create a Details render for when this grid-details render doesn’t fit
    /// in the terminal (or something has gone wrong) and we have given up, or
    /// when the user asked for a grid-details view but the terminal width is
    /// not available, so we downgrade.
    pub fn give_up(self) -> DetailsRender<'a> {
        DetailsRender {
            dir:           self.dir,
            files:         self.files,
            theme:         self.theme,
            file_style:    self.file_style,
            opts:          self.details,
            recurse:       None,
            filter:        self.filter,
            git_ignoring:  self.git_ignoring,
            git:           self.git,
        }
    }

    // This doesn’t take an IgnoreCache even though the details one does
    // because grid-details has no tree view.

    pub fn render<W: Write>(mut self, w: &mut W) -> io::Result<()> {
        if let Some((grid, width)) = self.find_fitting_grid() {
            write!(w, "{}", grid.fit_into_columns(width))
        }
        else {
            self.give_up().render(w)
        }
    }

    pub fn find_fitting_grid(&mut self) -> Option<(grid::Grid, grid::Width)> {
        let options = self.details.table.as_ref().expect("Details table options not given!");

        let drender = self.details_for_column();

        let (first_table, _) = self.make_table(options, &drender);

        let rows = self.files.iter()
                       .map(|file| first_table.row_for_file(file, file_has_xattrs(file)))
                       .collect::<Vec<_>>();

        let file_names = self.files.iter()
                             .map(|file| self.file_style.for_file(file, self.theme).paint().promote())
                             .collect::<Vec<_>>();

        let mut last_working_grid = self.make_grid(1, options, &file_names, rows.clone(), &drender);
        
        if file_names.len() == 1 {
            return Some((last_working_grid, 1));
        }

        // If we can’t fit everything in a grid 100 columns wide, then
        // something has gone seriously awry
        for column_count in 2..100 {
            let grid = self.make_grid(column_count, options, &file_names, rows.clone(), &drender);

            let the_grid_fits = {
                let d = grid.fit_into_columns(column_count);
                d.width() <= self.console_width
            };

            if the_grid_fits {
                last_working_grid = grid;
            }
            
            if !the_grid_fits || column_count == file_names.len() {
                let last_column_count = if the_grid_fits { column_count } else { column_count - 1 };
                // If we’ve figured out how many columns can fit in the user’s terminal,
                // and it turns out there aren’t enough rows to make it worthwhile
                // (according to EXA_GRID_ROWS), then just resort to the lines view.
                if let RowThreshold::MinimumRows(thresh) = self.row_threshold {
                    if last_working_grid.fit_into_columns(last_column_count).row_count() < thresh {
                        return None;
                    }
                }

                return Some((last_working_grid, last_column_count));
            }
        }

        None
    }

    fn make_table(&mut self, options: &'a TableOptions, drender: &DetailsRender<'_>) -> (Table<'a>, Vec<DetailsRow>) {
        match (self.git, self.dir) {
            (Some(g), Some(d))  => if ! g.has_anything_for(&d.path) { self.git = None },
            (Some(g), None)     => if ! self.files.iter().any(|f| g.has_anything_for(&f.path)) { self.git = None },
            (None,    _)        => {/* Keep Git how it is */},
        }

        let mut table = Table::new(options, self.git, &self.theme);
        let mut rows = Vec::new();

        if self.details.header {
            let row = table.header_row();
            table.add_widths(&row);
            rows.push(drender.render_header(row));
        }

        (table, rows)
    }

    fn make_grid(&mut self, column_count: usize, options: &'a TableOptions, file_names: &[TextCell], rows: Vec<TableRow>, drender: &DetailsRender<'_>) -> grid::Grid {
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

        let columns = tables
            .into_iter()
            .map(|(table, details_rows)| {
                drender.iterate_with_table(table, details_rows)
                       .collect::<Vec<_>>()
                })
            .collect::<Vec<_>>();

        let direction = if self.grid.across { grid::Direction::LeftToRight }
                                       else { grid::Direction::TopToBottom };

        let filling = grid::Filling::Spaces(4);
        let mut grid = grid::Grid::new(grid::GridOptions { direction, filling });

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

    if a % b != 0 {
        result += 1;
    }

    result
}


fn file_has_xattrs(file: &File<'_>) -> bool {
    match file.path.attributes() {
        Ok(attrs)  => ! attrs.is_empty(),
        Err(_)     => false,
    }
}
