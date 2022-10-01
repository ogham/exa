//! The **Details** output view displays each file as a row in a table.
//!
//! It’s used in the following situations:
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


use std::io::{self, Write};
use std::mem::MaybeUninit;
use std::path::PathBuf;
use std::vec::IntoIter as VecIntoIter;

use ansi_term::Style;
use scoped_threadpool::Pool;

use crate::fs::{Dir, File};
use crate::fs::dir_action::RecurseOptions;
use crate::fs::feature::git::GitCache;
use crate::fs::feature::xattr::{Attribute, FileAttributes};
use crate::fs::filter::FileFilter;
use crate::output::cell::TextCell;
use crate::output::file_name::Options as FileStyle;
use crate::output::table::{Table, Options as TableOptions, Row as TableRow};
use crate::output::tree::{TreeTrunk, TreeParams, TreeDepth};
use crate::theme::Theme;


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
#[derive(PartialEq, Eq, Debug)]
pub struct Options {

    /// Options specific to drawing a table.
    ///
    /// Directories themselves can pick which columns are *added* to this
    /// list, such as the Git column.
    pub table: Option<TableOptions>,

    /// Whether to show a header line or not.
    pub header: bool,

    /// Whether to show each file’s extended attributes.
    pub xattr: bool,
}


pub struct Render<'a> {
    pub dir: Option<&'a Dir>,
    pub files: Vec<File<'a>>,
    pub theme: &'a Theme,
    pub file_style: &'a FileStyle,
    pub opts: &'a Options,

    /// Whether to recurse through directories with a tree view, and if so,
    /// which options to use. This field is only relevant here if the `tree`
    /// field of the RecurseOptions is `true`.
    pub recurse: Option<RecurseOptions>,

    /// How to sort and filter the files after getting their details.
    pub filter: &'a FileFilter,

    /// Whether we are skipping Git-ignored files.
    pub git_ignoring: bool,

    pub git: Option<&'a GitCache>,
}


struct Egg<'a> {
    table_row: Option<TableRow>,
    xattrs:    Vec<Attribute>,
    errors:    Vec<(io::Error, Option<PathBuf>)>,
    dir:       Option<Dir>,
    file:      &'a File<'a>,
}

impl<'a> AsRef<File<'a>> for Egg<'a> {
    fn as_ref(&self) -> &File<'a> {
        self.file
    }
}


impl<'a> Render<'a> {
    pub fn render<W: Write>(mut self, w: &mut W) -> io::Result<()> {
        let n_cpus = match num_cpus::get() as u32 {
            0 => 1,
            n => n,
        };
        let mut pool = Pool::new(n_cpus);
        let mut rows = Vec::new();

        if let Some(ref table) = self.opts.table {
            match (self.git, self.dir) {
                (Some(g), Some(d))  => if ! g.has_anything_for(&d.path) { self.git = None },
                (Some(g), None)     => if ! self.files.iter().any(|f| g.has_anything_for(&f.path)) { self.git = None },
                (None,    _)        => {/* Keep Git how it is */},
            }

            let mut table = Table::new(table, self.git, self.theme);

            if self.opts.header {
                let header = table.header_row();
                table.add_widths(&header);
                rows.push(self.render_header(header));
            }

            // This is weird, but I can’t find a way around it:
            // https://internals.rust-lang.org/t/should-option-mut-t-implement-copy/3715/6
            let mut table = Some(table);
            self.add_files_to_table(&mut pool, &mut table, &mut rows, &self.files, TreeDepth::root());

            for row in self.iterate_with_table(table.unwrap(), rows) {
                writeln!(w, "{}", row.strings())?
            }
        }
        else {
            self.add_files_to_table(&mut pool, &mut None, &mut rows, &self.files, TreeDepth::root());

            for row in self.iterate(rows) {
                writeln!(w, "{}", row.strings())?
            }
        }

        Ok(())
    }

    /// Adds files to the table, possibly recursively. This is easily
    /// parallelisable, and uses a pool of threads.
    fn add_files_to_table<'dir>(&self, pool: &mut Pool, table: &mut Option<Table<'a>>, rows: &mut Vec<Row>, src: &[File<'dir>], depth: TreeDepth) {
        use std::sync::{Arc, Mutex};
        use log::*;
        use crate::fs::feature::xattr;

        let mut file_eggs = (0..src.len()).map(|_| MaybeUninit::uninit()).collect::<Vec<_>>();

        pool.scoped(|scoped| {
            let file_eggs = Arc::new(Mutex::new(&mut file_eggs));
            let table = table.as_ref();

            for (idx, file) in src.iter().enumerate() {
                let file_eggs = Arc::clone(&file_eggs);

                scoped.execute(move || {
                    let mut errors = Vec::new();
                    let mut xattrs = Vec::new();

                    // There are three “levels” of extended attribute support:
                    //
                    // 1. If we’re compiling without that feature, then
                    //    exa pretends all files have no attributes.
                    // 2. If the feature is enabled and the --extended flag
                    //    has been specified, then display an @ in the
                    //    permissions column for files with attributes, the
                    //    names of all attributes and their lengths, and any
                    //    errors encountered when getting them.
                    // 3. If the --extended flag *hasn’t* been specified, then
                    //    display the @, but don’t display anything else.
                    //
                    // For a while, exa took a stricter approach to (3):
                    // if an error occurred while checking a file’s xattrs to
                    // see if it should display the @, exa would display that
                    // error even though the attributes weren’t actually being
                    // shown! This was confusing, as users were being shown
                    // errors for something they didn’t explicitly ask for,
                    // and just cluttered up the output. So now errors aren’t
                    // printed unless the user passes --extended to signify
                    // that they want to see them.

                    if xattr::ENABLED {
                        match file.path.attributes() {
                            Ok(xs) => {
                                xattrs.extend(xs);
                            }
                            Err(e) => {
                                if self.opts.xattr {
                                    errors.push((e, None));
                                }
                                else {
                                    error!("Error looking up xattr for {:?}: {:#?}", file.path, e);
                                }
                            }
                        }
                    }

                    let table_row = table.as_ref()
                                         .map(|t| t.row_for_file(file, ! xattrs.is_empty()));

                    if ! self.opts.xattr {
                        xattrs.clear();
                    }

                    let mut dir = None;
                    if let Some(r) = self.recurse {
                        if file.is_directory() && r.tree && ! r.is_too_deep(depth.0) {
                            match file.to_dir() {
                                Ok(d) => {
                                    dir = Some(d);
                                }
                                Err(e) => {
                                    errors.push((e, None));
                                }
                            }
                        }
                    };

                    let egg = Egg { table_row, xattrs, errors, dir, file };
                    unsafe { std::ptr::write(file_eggs.lock().unwrap()[idx].as_mut_ptr(), egg) }
                });
            }
        });

        // this is safe because all entries have been initialized above
        let mut file_eggs = unsafe { std::mem::transmute::<_, Vec<Egg<'_>>>(file_eggs) };
        self.filter.sort_files(&mut file_eggs);

        for (tree_params, egg) in depth.iterate_over(file_eggs.into_iter()) {
            let mut files = Vec::new();
            let mut errors = egg.errors;

            if let (Some(ref mut t), Some(row)) = (table.as_mut(), egg.table_row.as_ref()) {
                t.add_widths(row);
            }

            let file_name = self.file_style.for_file(egg.file, self.theme)
                                .with_link_paths()
                                .paint()
                                .promote();

            let row = Row {
                tree:   tree_params,
                cells:  egg.table_row,
                name:   file_name,
            };

            rows.push(row);

            if let Some(ref dir) = egg.dir {
                for file_to_add in dir.files(self.filter.dot_filter, self.git, self.git_ignoring) {
                    match file_to_add {
                        Ok(f) => {
                            files.push(f);
                        }
                        Err((path, e)) => {
                            errors.push((e, Some(path)));
                        }
                    }
                }

                self.filter.filter_child_files(&mut files);

                if ! files.is_empty() {
                    for xattr in egg.xattrs {
                        rows.push(self.render_xattr(&xattr, TreeParams::new(depth.deeper(), false)));
                    }

                    for (error, path) in errors {
                        rows.push(self.render_error(&error, TreeParams::new(depth.deeper(), false), path));
                    }

                    self.add_files_to_table(pool, table, rows, &files, depth.deeper());
                    continue;
                }
            }

            let count = egg.xattrs.len();
            for (index, xattr) in egg.xattrs.into_iter().enumerate() {
                let params = TreeParams::new(depth.deeper(), errors.is_empty() && index == count - 1);
                let r = self.render_xattr(&xattr, params);
                rows.push(r);
            }

            let count = errors.len();
            for (index, (error, path)) in errors.into_iter().enumerate() {
                let params = TreeParams::new(depth.deeper(), index == count - 1);
                let r = self.render_error(&error, params, path);
                rows.push(r);
            }
        }
    }

    pub fn render_header(&self, header: TableRow) -> Row {
        Row {
            tree:     TreeParams::new(TreeDepth::root(), false),
            cells:    Some(header),
            name:     TextCell::paint_str(self.theme.ui.header, "Name"),
        }
    }

    fn render_error(&self, error: &io::Error, tree: TreeParams, path: Option<PathBuf>) -> Row {
        use crate::output::file_name::Colours;

        let error_message = if let Some(path) = path {
            format!("<{}: {}>", path.display(), error)
        } else {
            format!("<{}>", error)
        };

        // TODO: broken_symlink() doesn’t quite seem like the right name for
        // the style that’s being used here. Maybe split it in two?
        let name = TextCell::paint(self.theme.broken_symlink(), error_message);
        Row { cells: None, name, tree }
    }

    fn render_xattr(&self, xattr: &Attribute, tree: TreeParams) -> Row {
        let name = TextCell::paint(self.theme.ui.perms.attribute, format!("{} (len {})", xattr.name, xattr.size));
        Row { cells: None, name, tree }
    }

    pub fn render_file(&self, cells: TableRow, name: TextCell, tree: TreeParams) -> Row {
        Row { cells: Some(cells), name, tree }
    }

    pub fn iterate_with_table(&'a self, table: Table<'a>, rows: Vec<Row>) -> TableIter<'a> {
        TableIter {
            tree_trunk: TreeTrunk::default(),
            total_width: table.widths().total(),
            table,
            inner: rows.into_iter(),
            tree_style: self.theme.ui.punctuation,
        }
    }

    pub fn iterate(&'a self, rows: Vec<Row>) -> Iter {
        Iter {
            tree_trunk: TreeTrunk::default(),
            inner: rows.into_iter(),
            tree_style: self.theme.ui.punctuation,
        }
    }
}


pub struct Row {

    /// Vector of cells to display.
    ///
    /// Most of the rows will be used to display files’ metadata, so this will
    /// almost always be `Some`, containing a vector of cells. It will only be
    /// `None` for a row displaying an attribute or error, neither of which
    /// have cells.
    pub cells: Option<TableRow>,

    /// This file’s name, in coloured output. The name is treated separately
    /// from the other cells, as it never requires padding.
    pub name: TextCell,

    /// Information used to determine which symbols to display in a tree.
    pub tree: TreeParams,
}


pub struct TableIter<'a> {
    inner: VecIntoIter<Row>,
    table: Table<'a>,

    total_width: usize,
    tree_style:  Style,
    tree_trunk:  TreeTrunk,
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
                cell.push(self.tree_style.paint(tree_part.ascii_art()), 4);
            }

            // If any tree characters have been printed, then add an extra
            // space, which makes the output look much better.
            if ! row.tree.is_at_root() {
                cell.add_spaces(1);
            }

            cell.append(row.name);
            cell
        })
    }
}


pub struct Iter {
    tree_trunk: TreeTrunk,
    tree_style: Style,
    inner: VecIntoIter<Row>,
}

impl Iterator for Iter {
    type Item = TextCell;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|row| {
            let mut cell = TextCell::default();

            for tree_part in self.tree_trunk.new_row(row.tree) {
                cell.push(self.tree_style.paint(tree_part.ascii_art()), 4);
            }

            // If any tree characters have been printed, then add an extra
            // space, which makes the output look much better.
            if ! row.tree.is_at_root() {
                cell.add_spaces(1);
            }

            cell.append(row.name);
            cell
        })
    }
}
