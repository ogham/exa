//! The **Details** output view displays each file as a row in a table.
//!
//! It's used in the following situations:
//!
//! - Most commonly, when using the `--long` command-line argument to display the
//!   details of each file, which requires using a table view to hold all the data;
//! - When using the `--tree` argument, which uses the same table view to display
//!   each file on its own line, with the table providing the tree characters;
//! - When using both the `--long` and `--grid` arguments, which constructs a
//!   series of tables to fit all the data on the screen.
//!
//! You will probably recognise it from the `ls --long` command. It looks like
//! this:
//!
//! ```text
//!     .rw-r--r--  9.6k ben 29 Jun 16:16 Cargo.lock
//!     .rw-r--r--   547 ben 23 Jun 10:54 Cargo.toml
//!     .rw-r--r--  1.1k ben 23 Nov  2014 LICENCE
//!     .rw-r--r--  2.5k ben 21 May 14:38 README.md
//!     .rw-r--r--  382k ben  8 Jun 21:00 screenshot.png
//!     drwxr-xr-x     - ben 29 Jun 14:50 src
//!     drwxr-xr-x     - ben 28 Jun 19:53 target
//! ```
//!
//! The table is constructed by creating a `Table` value, which produces a `Row`
//! value for each file. These rows can contain a vector of `Cell`s, or they can
//! contain depth information for the tree view, or both. These are described
//! below.
//!
//!
//! ## Constructing Detail Views
//!
//! When using the `--long` command-line argument, the details of each file are
//! displayed next to its name.
//!
//! The table holds a vector of all the column types. For each file and column, a
//! `Cell` value containing the ANSI-coloured text and Unicode width of each cell
//! is generated, with the row and column determined by indexing into both arrays.
//!
//! The column types vector does not actually include the filename. This is
//! because the filename is always the rightmost field, and as such, it does not
//! need to have its width queried or be padded with spaces.
//!
//! To illustrate the above:
//!
//! ```text
//!     ┌─────────────────────────────────────────────────────────────────────────┐
//!     │ columns: [ Permissions,  Size,   User,  Date(Modified) ]                │
//!     ├─────────────────────────────────────────────────────────────────────────┤
//!     │   rows:  cells:                                            filename:    │
//!     │   row 1: [ ".rw-r--r--", "9.6k", "ben", "29 Jun 16:16" ]   Cargo.lock   │
//!     │   row 2: [ ".rw-r--r--",  "547", "ben", "23 Jun 10:54" ]   Cargo.toml   │
//!     │   row 3: [ "drwxr-xr-x",    "-", "ben", "29 Jun 14:50" ]   src          │
//!     │   row 4: [ "drwxr-xr-x",    "-", "ben", "28 Jun 19:53" ]   target       │
//!     └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! Each column in the table needs to be resized to fit its widest argument. This
//! means that we must wait until every row has been added to the table before it
//! can be displayed, in order to make sure that every column is wide enough.


use std::io::{Write, Error as IOError, Result as IOResult};
use std::path::PathBuf;
use std::vec::IntoIter as VecIntoIter;

use fs::{Dir, File};
use fs::feature::xattr::{Attribute, FileAttributes};
use options::{FileFilter, RecurseOptions};
use output::colours::Colours;
use output::column::Columns;
use output::cell::TextCell;
use output::tree::{TreeTrunk, TreeParams, TreeDepth};
use output::file_name::{FileName, LinkStyle, Classify};
use output::table::{Table, Environment, Row as TableRow};


/// With the **Details** view, the output gets formatted into columns, with
/// each `Column` object showing some piece of information about the file,
/// such as its size, or its permissions.
///
/// To do this, the results have to be written to a table, instead of
/// displaying each file immediately. Then, the width of each column can be
/// calculated based on the individual results, and the fields are padded
/// during output.
///
/// Almost all the heavy lifting is done in a Table object, which handles the
/// columns for each row.
#[derive(PartialEq, Debug, Clone, Default)]
pub struct Options {

    /// A Columns object that says which columns should be included in the
    /// output in the general case. Directories themselves can pick which
    /// columns are *added* to this list, such as the Git column.
    pub columns: Option<Columns>,

    /// Whether to show a header line or not.
    pub header: bool,

    /// Whether to show each file's extended attributes.
    pub xattr: bool,
}



pub struct Render<'a> {
    pub dir: Option<&'a Dir>,
    pub files: Vec<File<'a>>,
    pub colours: &'a Colours,
    pub classify: Classify,
    pub opts: &'a Options,

    /// Whether to recurse through directories with a tree view, and if so,
    /// which options to use. This field is only relevant here if the `tree`
    /// field of the RecurseOptions is `true`.
    pub recurse: Option<RecurseOptions>,

    /// How to sort and filter the files after getting their details.
    pub filter: &'a FileFilter,
}


struct Egg<'a> {
    table_row: Option<TableRow>,
    xattrs:    Vec<Attribute>,
    errors:    Vec<(IOError, Option<PathBuf>)>,
    dir:       Option<Dir>,
    file:      &'a File<'a>,
}

impl<'a> AsRef<File<'a>> for Egg<'a> {
    fn as_ref(&self) -> &File<'a> {
        self.file
    }
}


impl<'a> Render<'a> {
    pub fn render<W: Write>(self, w: &mut W) -> IOResult<()> {
        let mut rows = Vec::new();

        if let Some(columns) = self.opts.columns {
            let env = Environment::default();
            let colz = columns.for_dir(self.dir);
            let mut table = Table::new(&colz, &self.colours, &env);

            if self.opts.header {
                let header = table.header_row();
                table.add_widths(&header);
                rows.push(self.render_header(header));
            }

            // This is weird, but I can't find a way around it:
            // https://internals.rust-lang.org/t/should-option-mut-t-implement-copy/3715/6
            let mut table = Some(table);
            self.add_files_to_table(&mut table, &mut rows, &self.files, TreeDepth::root());

            for row in self.iterate_with_table(table.unwrap(), rows) {
                writeln!(w, "{}", row.strings())?
            }
        }
        else {
            self.add_files_to_table(&mut None, &mut rows, &self.files, TreeDepth::root());

            for row in self.iterate(rows) {
                writeln!(w, "{}", row.strings())?
            }
        }

        Ok(())
    }

    /// Adds files to the table, possibly recursively. This is easily
    /// parallelisable, and uses a pool of threads.
    fn add_files_to_table<'dir>(&self, table: &mut Option<Table<'a>>, rows: &mut Vec<Row>, src: &Vec<File<'dir>>, depth: TreeDepth) {
        use num_cpus;
        use scoped_threadpool::Pool;
        use std::sync::{Arc, Mutex};
        use fs::feature::xattr;

        let mut pool = Pool::new(num_cpus::get() as u32);
        let mut file_eggs = Vec::new();

        pool.scoped(|scoped| {
            let file_eggs = Arc::new(Mutex::new(&mut file_eggs));
            let table = table.as_ref();

            for file in src {
                let file_eggs = file_eggs.clone();

                scoped.execute(move || {
                    let mut errors = Vec::new();
                    let mut xattrs = Vec::new();

                    if xattr::ENABLED {
                        match file.path.attributes() {
                            Ok(xs) => xattrs.extend(xs),
                            Err(e) => errors.push((e, None)),
                        };
                    }

                    let table_row = table.as_ref().map(|t| t.row_for_file(&file, !xattrs.is_empty()));

                    if !self.opts.xattr {
                        xattrs.clear();
                    }

                    let mut dir = None;

                    if let Some(r) = self.recurse {
                        if file.is_directory() && r.tree && !r.is_too_deep(depth.0) {
                            if let Ok(d) = file.to_dir(false) {
                                dir = Some(d);
                            }
                        }
                    };

                    let egg = Egg { table_row, xattrs, errors, dir, file };
                    file_eggs.lock().unwrap().push(egg);
                });
            }
        });

        self.filter.sort_files(&mut file_eggs);

        let num_eggs = file_eggs.len();
        for (index, egg) in file_eggs.into_iter().enumerate() {
            let mut files = Vec::new();
            let mut errors = egg.errors;

            if let (Some(ref mut t), Some(ref row)) = (table.as_mut(), egg.table_row.as_ref()) {
                t.add_widths(row);
            }

            let row = Row {
                tree:   TreeParams::new(depth, index == num_eggs - 1),
                cells:  egg.table_row,
                name:   FileName::new(&egg.file, LinkStyle::FullLinkPaths, self.classify, self.colours).paint().promote(),
            };

            rows.push(row);

            if let Some(ref dir) = egg.dir {
                for file_to_add in dir.files(self.filter.dot_filter) {
                    match file_to_add {
                        Ok(f)          => files.push(f),
                        Err((path, e)) => errors.push((e, Some(path)))
                    }
                }

                self.filter.filter_child_files(&mut files);

                if !files.is_empty() {
                    for xattr in egg.xattrs {
                        rows.push(self.render_xattr(xattr, TreeParams::new(depth.deeper(), false)));
                    }

                    for (error, path) in errors {
                        rows.push(self.render_error(&error, TreeParams::new(depth.deeper(), false), path));
                    }

                    self.add_files_to_table(table, rows, &files, depth.deeper());
                    continue;
                }
            }

            let count = egg.xattrs.len();
            for (index, xattr) in egg.xattrs.into_iter().enumerate() {
                rows.push(self.render_xattr(xattr, TreeParams::new(depth.deeper(), errors.is_empty() && index == count - 1)));
            }

            let count = errors.len();
            for (index, (error, path)) in errors.into_iter().enumerate() {
                rows.push(self.render_error(&error, TreeParams::new(depth.deeper(), index == count - 1), path));
            }
        }
    }

    pub fn render_header(&self, header: TableRow) -> Row {
        Row {
            tree:     TreeParams::new(TreeDepth::root(), false),
            cells:    Some(header),
            name:     TextCell::paint_str(self.colours.header, "Name"),
        }
    }

    fn render_error(&self, error: &IOError, tree: TreeParams, path: Option<PathBuf>) -> Row {
        let error_message = match path {
            Some(path) => format!("<{}: {}>", path.display(), error),
            None       => format!("<{}>", error),
        };

        let name = TextCell::paint(self.colours.broken_arrow, error_message);
        Row { cells: None, name, tree }
    }

    fn render_xattr(&self, xattr: Attribute, tree: TreeParams) -> Row {
        let name = TextCell::paint(self.colours.perms.attribute, format!("{} (len {})", xattr.name, xattr.size));
        Row { cells: None, name, tree }
    }

    pub fn render_file(&self, cells: TableRow, name: TextCell, tree: TreeParams) -> Row {
        Row { cells: Some(cells), name, tree }
    }

    pub fn iterate_with_table(&'a self, table: Table<'a>, rows: Vec<Row>) -> TableIter<'a> {
        TableIter {
            tree_trunk: TreeTrunk::default(),
            total_width: table.widths().total(),
            table: table,
            inner: rows.into_iter(),
            colours: self.colours,
        }
    }

    pub fn iterate(&'a self, rows: Vec<Row>) -> Iter<'a> {
        Iter {
            tree_trunk: TreeTrunk::default(),
            inner: rows.into_iter(),
            colours: self.colours,
        }
    }
}


pub struct Row {

    /// Vector of cells to display.
    ///
    /// Most of the rows will be used to display files' metadata, so this will
    /// almost always be `Some`, containing a vector of cells. It will only be
    /// `None` for a row displaying an attribute or error, neither of which
    /// have cells.
    pub cells: Option<TableRow>,

    /// This file's name, in coloured output. The name is treated separately
    /// from the other cells, as it never requires padding.
    pub name: TextCell,

    /// Information used to determine which symbols to display in a tree.
    pub tree: TreeParams,
}


pub struct TableIter<'a> {
    table: Table<'a>,
    tree_trunk: TreeTrunk,
    total_width: usize,
    colours: &'a Colours,
    inner: VecIntoIter<Row>,
}

impl<'a> Iterator for TableIter<'a> {
    type Item = TextCell;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|row| {
            let mut cell =
                if let Some(cells) = row.cells {
                    self.table.render(cells)
                }
                else {
                    let mut cell = TextCell::default();
                    cell.add_spaces(self.total_width);
                    cell
                };

            for tree_part in self.tree_trunk.new_row(row.tree) {
                cell.push(self.colours.punctuation.paint(tree_part.ascii_art()), 4);
            }

            // If any tree characters have been printed, then add an extra
            // space, which makes the output look much better.
            if !row.tree.is_at_root() {
                cell.add_spaces(1);
            }

            cell.append(row.name);
            cell
        })
    }
}


pub struct Iter<'a> {
    tree_trunk: TreeTrunk,
    colours: &'a Colours,
    inner: VecIntoIter<Row>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = TextCell;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|row| {
            let mut cell = TextCell::default();

            for tree_part in self.tree_trunk.new_row(row.tree) {
                cell.push(self.colours.punctuation.paint(tree_part.ascii_art()), 4);
            }

            // If any tree characters have been printed, then add an extra
            // space, which makes the output look much better.
            if !row.tree.is_at_root() {
                cell.add_spaces(1);
            }

            cell.append(row.name);
            cell
        })
    }
}
