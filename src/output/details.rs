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
//!
//!
//! ## Extended Attributes and Errors
//!
//! Finally, files' extended attributes and any errors that occur while statting
//! them can also be displayed as their children. It looks like this:
//!
//! ```text
//!     .rw-r--r--  0 ben  3 Sep 13:26 forbidden
//!                                    └── <Permission denied (os error 13)>
//!     .rw-r--r--@ 0 ben  3 Sep 13:26 file_with_xattrs
//!                                    ├── another_greeting (len 2)
//!                                    └── greeting (len 5)
//! ```
//!
//! These lines also have `None` cells, and the error string or attribute details
//! are used in place of the filename.


use std::io::{Write, Error as IOError, Result as IOResult};
use std::ops::Add;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use datetime::fmt::DateFormat;
use datetime::{LocalDateTime, DatePiece};
use datetime::TimeZone;
use zoneinfo_compiled::{CompiledData, Result as TZResult};

use locale;

use users::{Users, Groups, UsersCache};

use fs::{Dir, File, fields as f};
use fs::feature::xattr::{Attribute, FileAttributes};
use options::{FileFilter, RecurseOptions};
use output::colours::Colours;
use output::column::{Alignment, Column, Columns};
use output::cell::{TextCell, TextCellContents};
use output::tree::TreeTrunk;
use output::file_name::{FileName, LinkStyle, Classify};


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
pub struct Details {

    /// A Columns object that says which columns should be included in the
    /// output in the general case. Directories themselves can pick which
    /// columns are *added* to this list, such as the Git column.
    pub columns: Option<Columns>,

    /// Whether to recurse through directories with a tree view, and if so,
    /// which options to use. This field is only relevant here if the `tree`
    /// field of the RecurseOptions is `true`.
    pub recurse: Option<RecurseOptions>,

    /// How to sort and filter the files after getting their details.
    pub filter: FileFilter,

    /// Whether to show a header line or not.
    pub header: bool,

    /// Whether to show each file's extended attributes.
    pub xattr: bool,

    /// The colours to use to display information in the table, including the
    /// colour of the tree view symbols.
    pub colours: Colours,

    /// Whether to show a file type indiccator.
    pub classify: Classify,
}

/// The **environment** struct contains any data that could change between
/// running instances of exa, depending on the user's computer's configuration.
///
/// Any environment field should be able to be mocked up for test runs.
pub struct Environment<U> {  // where U: Users+Groups

    /// The year of the current time. This gets used to determine which date
    /// format to use.
    current_year: i64,

    /// Localisation rules for formatting numbers.
    numeric: locale::Numeric,

    /// Localisation rules for formatting timestamps.
    time: locale::Time,

    /// Date format for printing out timestamps that are in the current year.
    date_and_time: DateFormat<'static>,

    /// Date format for printing out timestamps that *aren’t*.
    date_and_year: DateFormat<'static>,

    /// The computer's current time zone. This gets used to determine how to
    /// offset files' timestamps.
    tz: Option<TimeZone>,

    /// Mapping cache of user IDs to usernames.
    users: Mutex<U>,
}

impl<U> Environment<U> {
    pub fn lock_users(&self) -> MutexGuard<U> {
        self.users.lock().unwrap()
    }
}

impl Default for Environment<UsersCache> {
    fn default() -> Self {
        use unicode_width::UnicodeWidthStr;

        let tz = determine_time_zone();
        if let Err(ref e) = tz {
            println!("Unable to determine time zone: {}", e);
        }

        let numeric = locale::Numeric::load_user_locale()
                          .unwrap_or_else(|_| locale::Numeric::english());

        let time = locale::Time::load_user_locale()
                       .unwrap_or_else(|_| locale::Time::english());

        // Some locales use a three-character wide month name (Jan to Dec);
        // others vary between three and four (1月 to 12月). We assume that
        // December is the month with the maximum width, and use the width of
        // that to determine how to pad the other months.
        let december_width = UnicodeWidthStr::width(&*time.short_month_name(11));
        let date_and_time = match december_width {
            4  => DateFormat::parse("{2>:D} {4>:M} {2>:h}:{02>:m}").unwrap(),
            _  => DateFormat::parse("{2>:D} {:M} {2>:h}:{02>:m}").unwrap(),
        };

        let date_and_year = match december_width {
            4 => DateFormat::parse("{2>:D} {4>:M} {5>:Y}").unwrap(),
            _ => DateFormat::parse("{2>:D} {:M} {5>:Y}").unwrap()
        };

        Environment {
            current_year:  LocalDateTime::now().year(),
            numeric:       numeric,
            date_and_time: date_and_time,
            date_and_year: date_and_year,
            time:          time,
            tz:            tz.ok(),
            users:         Mutex::new(UsersCache::new()),
        }
    }
}

fn determine_time_zone() -> TZResult<TimeZone> {
    TimeZone::from_file("/etc/localtime")
}

impl Details {

    /// Print the details of the given vector of files -- all of which will
    /// have been read from the given directory, if present -- to stdout.
    pub fn view<W: Write>(&self, dir: Option<&Dir>, files: Vec<File>, w: &mut W) -> IOResult<()> {

        // First, transform the Columns object into a vector of columns for
        // the current directory.
        let columns_for_dir = match self.columns {
            Some(cols) => cols.for_dir(dir),
            None => Vec::new(),
        };

        // Then, retrieve various environment variables.
        let env = Arc::new(Environment::<UsersCache>::default());

        // Build the table to put rows in.
        let mut table = Table {
            columns: &*columns_for_dir,
            opts: self,
            env: env,
            rows: Vec::new(),
        };

        // Next, add a header if the user requests it.
        if self.header { table.add_header() }

        // Then add files to the table and print it out.
        self.add_files_to_table(&mut table, files, 0);
        for cell in table.print_table() {
            writeln!(w, "{}", cell.strings())?;
        }

        Ok(())
    }

    /// Adds files to the table, possibly recursively. This is easily
    /// parallelisable, and uses a pool of threads.
    fn add_files_to_table<'dir, U: Users+Groups+Send>(&self, mut table: &mut Table<U>, src: Vec<File<'dir>>, depth: usize) {
        use num_cpus;
        use scoped_threadpool::Pool;
        use std::sync::{Arc, Mutex};
        use fs::feature::xattr;

        let mut pool = Pool::new(num_cpus::get() as u32);
        let mut file_eggs = Vec::new();

        struct Egg<'a> {
            cells:   Vec<TextCell>,
            xattrs:  Vec<Attribute>,
            errors:  Vec<(IOError, Option<PathBuf>)>,
            dir:     Option<Dir>,
            file:    File<'a>,
        }

        impl<'a> AsRef<File<'a>> for Egg<'a> {
            fn as_ref(&self) -> &File<'a> {
                &self.file
            }
        }

        pool.scoped(|scoped| {
            let file_eggs = Arc::new(Mutex::new(&mut file_eggs));
            let table = Arc::new(&mut table);

            for file in src {
                let file_eggs = file_eggs.clone();
                let table = table.clone();

                scoped.execute(move || {
                    let mut errors = Vec::new();
                    let mut xattrs = Vec::new();

                    if xattr::ENABLED {
                        match file.path.attributes() {
                            Ok(xs) => xattrs.extend(xs),
                            Err(e) => errors.push((e, None)),
                        };
                    }

                    let cells = table.cells_for_file(&file, !xattrs.is_empty());

                    if !table.opts.xattr {
                        xattrs.clear();
                    }

                    let mut dir = None;

                    if let Some(r) = self.recurse {
                        if file.is_directory() && r.tree && !r.is_too_deep(depth) {
                            if let Ok(d) = file.to_dir(false) {
                                dir = Some(d);
                            }
                        }
                    };

                    let egg = Egg { cells, xattrs, errors, dir, file };
                    file_eggs.lock().unwrap().push(egg);
                });
            }
        });

        self.filter.sort_files(&mut file_eggs);

        let num_eggs = file_eggs.len();
        for (index, egg) in file_eggs.into_iter().enumerate() {
            let mut files = Vec::new();
            let mut errors = egg.errors;

            let row = Row {
                depth:    depth,
                cells:    Some(egg.cells),
                name:     FileName::new(&egg.file, LinkStyle::FullLinkPaths, self.classify, &self.colours).paint().promote(),
                last:     index == num_eggs - 1,
            };

            table.rows.push(row);

            if let Some(ref dir) = egg.dir {
                for file_to_add in dir.files() {
                    match file_to_add {
                        Ok(f)          => files.push(f),
                        Err((path, e)) => errors.push((e, Some(path)))
                    }
                }

                self.filter.filter_child_files(&mut files);

                if !files.is_empty() {
                    for xattr in egg.xattrs {
                        table.add_xattr(xattr, depth + 1, false);
                    }

                    for (error, path) in errors {
                        table.add_error(&error, depth + 1, false, path);
                    }

                    self.add_files_to_table(table, files, depth + 1);
                    continue;
                }
            }

            let count = egg.xattrs.len();
            for (index, xattr) in egg.xattrs.into_iter().enumerate() {
                table.add_xattr(xattr, depth + 1, errors.is_empty() && index == count - 1);
            }

            let count = errors.len();
            for (index, (error, path)) in errors.into_iter().enumerate() {
                table.add_error(&error, depth + 1, index == count - 1, path);
            }
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
    cells: Option<Vec<TextCell>>,

    /// This file's name, in coloured output. The name is treated separately
    /// from the other cells, as it never requires padding.
    name: TextCell,

    /// How many directories deep into the tree structure this is. Directories
    /// on top have depth 0.
    depth: usize,

    /// Whether this is the last entry in the directory. This flag is used
    /// when calculating the tree view.
    last: bool,
}

impl Row {

    /// Gets the Unicode display width of the indexed column, if present. If
    /// not, returns 0.
    fn column_width(&self, index: usize) -> usize {
        match self.cells {
            Some(ref cells) => *cells[index].width,
            None => 0,
        }
    }
}


/// A **Table** object gets built up by the view as it lists files and
/// directories.
pub struct Table<'a, U: 'a> { // where U: Users+Groups
    pub rows: Vec<Row>,

    pub columns: &'a [Column],
    pub opts: &'a Details,
    pub env: Arc<Environment<U>>,
}

impl<'a, U: Users+Groups+'a> Table<'a, U> {

    /// Add a dummy "header" row to the table, which contains the names of all
    /// the columns, underlined. This has dummy data for the cases that aren't
    /// actually used, such as the depth or list of attributes.
    pub fn add_header(&mut self) {
        let row = Row {
            depth:    0,
            cells:    Some(self.columns.iter().map(|c| TextCell::paint_str(self.opts.colours.header, c.header())).collect()),
            name:     TextCell::paint_str(self.opts.colours.header, "Name"),
            last:     false,
        };

        self.rows.push(row);
    }

    fn add_error(&mut self, error: &IOError, depth: usize, last: bool, path: Option<PathBuf>) {
        let error_message = match path {
            Some(path) => format!("<{}: {}>", path.display(), error),
            None       => format!("<{}>", error),
        };

        let row = Row {
            depth:    depth,
            cells:    None,
            name:     TextCell::paint(self.opts.colours.broken_arrow, error_message),
            last:     last,
        };

        self.rows.push(row);
    }

    fn add_xattr(&mut self, xattr: Attribute, depth: usize, last: bool) {
        let row = Row {
            depth:    depth,
            cells:    None,
            name:     TextCell::paint(self.opts.colours.perms.attribute, format!("{} (len {})", xattr.name, xattr.size)),
            last:     last,
        };

        self.rows.push(row);
    }

    pub fn filename(&self, file: File, links: LinkStyle) -> TextCellContents {
        FileName::new(&file, links, self.opts.classify, &self.opts.colours).paint()
    }

    pub fn add_file_with_cells(&mut self, cells: Vec<TextCell>, name_cell: TextCell, depth: usize, last: bool) {
        let row = Row {
            depth:  depth,
            cells:  Some(cells),
            name:   name_cell,
            last:   last,
        };

        self.rows.push(row);
    }

    /// Use the list of columns to find which cells should be produced for
    /// this file, per-column.
    pub fn cells_for_file(&self, file: &File, xattrs: bool) -> Vec<TextCell> {
        self.columns.iter()
                    .map(|c| self.display(file, c, xattrs))
                    .collect()
    }

    fn permissions_plus(&self, file: &File, xattrs: bool) -> f::PermissionsPlus {
        f::PermissionsPlus {
            file_type: file.type_char(),
            permissions: file.permissions(),
            xattrs: xattrs,
        }
    }

    fn display(&self, file: &File, column: &Column, xattrs: bool) -> TextCell {
        use output::column::TimeType::*;

        match *column {
            Column::Permissions          => self.permissions_plus(file, xattrs).render(&self.opts.colours),
            Column::FileSize(fmt)        => file.size().render(&self.opts.colours, fmt, &self.env.numeric),
            Column::Timestamp(Modified)  => file.modified_time().render(&self.opts.colours, &self.env.tz, &self.env.date_and_time, &self.env.date_and_year, &self.env.time, self.env.current_year),
            Column::Timestamp(Created)   => file.created_time().render( &self.opts.colours, &self.env.tz, &self.env.date_and_time, &self.env.date_and_year, &self.env.time, self.env.current_year),
            Column::Timestamp(Accessed)  => file.accessed_time().render(&self.opts.colours, &self.env.tz, &self.env.date_and_time, &self.env.date_and_year, &self.env.time, self.env.current_year),
            Column::HardLinks            => file.links().render(&self.opts.colours, &self.env.numeric),
            Column::Inode                => file.inode().render(&self.opts.colours),
            Column::Blocks               => file.blocks().render(&self.opts.colours),
            Column::User                 => file.user().render(&self.opts.colours, &*self.env.lock_users()),
            Column::Group                => file.group().render(&self.opts.colours, &*self.env.lock_users()),
            Column::GitStatus            => file.git_status().render(&self.opts.colours),
        }
    }

    /// Render the table as a vector of Cells, to be displayed on standard output.
    pub fn print_table(self) -> Vec<TextCell> {
        let mut tree_trunk = TreeTrunk::default();
        let mut cells = Vec::new();

        // Work out the list of column widths by finding the longest cell for
        // each column, then formatting each cell in that column to be the
        // width of that one.
        let column_widths: Vec<usize> = (0 .. self.columns.len())
            .map(|n| self.rows.iter().map(|row| row.column_width(n)).max().unwrap_or(0))
            .collect();

        let total_width: usize = self.columns.len() + column_widths.iter().fold(0, Add::add);

        for row in self.rows {
            let mut cell = TextCell::default();

            if let Some(cells) = row.cells {
                for (n, (this_cell, width)) in cells.into_iter().zip(column_widths.iter()).enumerate() {
                    let padding = width - *this_cell.width;

                    match self.columns[n].alignment() {
                        Alignment::Left  => { cell.append(this_cell); cell.add_spaces(padding); }
                        Alignment::Right => { cell.add_spaces(padding); cell.append(this_cell); }
                    }

                    cell.add_spaces(1);
                }
            }
            else {
                cell.add_spaces(total_width)
            }

            let mut filename = TextCell::default();

            for tree_part in tree_trunk.new_row(row.depth, row.last) {
                filename.push(self.opts.colours.punctuation.paint(tree_part.ascii_art()), 4);
            }

            // If any tree characters have been printed, then add an extra
            // space, which makes the output look much better.
            if row.depth != 0 {
                filename.add_spaces(1);
            }

            // Print the name without worrying about padding.
            filename.append(row.name);

            cell.append(filename);
            cells.push(cell);
        }

        cells
    }
}
