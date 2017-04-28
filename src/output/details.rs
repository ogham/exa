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
use std::string::ToString;
use std::sync::{Arc, Mutex};

use ansi_term::Style;

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
use output::column::{Alignment, Column, Columns, SizeFormat};
use output::cell::{TextCell, DisplayWidth};
use output::tree::TreeTrunk;
use super::filename;


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
    pub classify: bool,
}

/// The **environment** struct contains any data that could change between
/// running instances of exa, depending on the user's computer's configuration.
///
/// Any environment field should be able to be mocked up for test runs.
pub struct Environment<U: Users+Groups> {

    /// The year of the current time. This gets used to determine which date
    /// format to use.
    current_year: i64,

    /// Localisation rules for formatting numbers.
    numeric: locale::Numeric,

    /// Localisation rules for formatting timestamps.
    time: locale::Time,

    /// The computer's current time zone. This gets used to determine how to
    /// offset files' timestamps.
    tz: Option<TimeZone>,

    /// Mapping cache of user IDs to usernames.
    users: Mutex<U>,
}

impl Default for Environment<UsersCache> {
    fn default() -> Self {
        let tz = determine_time_zone();

        if let Err(ref e) = tz {
            println!("Unable to determine time zone: {}", e);
        }

        Environment {
            current_year: LocalDateTime::now().year(),
            numeric:      locale::Numeric::load_user_locale().unwrap_or_else(|_| locale::Numeric::english()),
            time:         locale::Time::load_user_locale().unwrap_or_else(|_| locale::Time::english()),
            tz:           tz.ok(),
            users:        Mutex::new(UsersCache::new()),
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

                    let egg = Egg {
                        cells: cells,
                        xattrs: xattrs,
                        errors: errors,
                        dir: dir,
                        file: file,
                    };

                    file_eggs.lock().unwrap().push(egg);
                });
            }
        });

        self.filter.sort_files(&mut file_eggs);

        let num_eggs = file_eggs.len();
        for (index, egg) in file_eggs.into_iter().enumerate() {
            let mut files = Vec::new();
            let mut errors = egg.errors;
            let mut width = DisplayWidth::from_file(&egg.file, self.classify);

            if egg.file.dir.is_none() {
                if let Some(parent) = egg.file.path.parent() {
                    width = width + 1 + DisplayWidth::from(parent.to_string_lossy().as_ref());
                }
            }

            let name = TextCell {
                contents: filename(&egg.file, &self.colours, true, self.classify),
                width:    width,
            };

            let row = Row {
                depth:    depth,
                cells:    Some(egg.cells),
                name:     name,
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
pub struct Table<'a, U: Users+Groups+'a> {
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

    pub fn filename_cell(&self, file: File, links: bool) -> TextCell {
        let mut width = DisplayWidth::from_file(&file, self.opts.classify);

        if file.dir.is_none() {
            if let Some(parent) = file.path.parent() {
                width = width + 1 + DisplayWidth::from(parent.to_string_lossy().as_ref());
            }
        }

        TextCell {
            contents: filename(&file, &self.opts.colours, links, self.opts.classify),
            width:    width,
        }
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

    fn display(&self, file: &File, column: &Column, xattrs: bool) -> TextCell {
        use output::column::TimeType::*;

        match *column {
            Column::Permissions          => self.render_permissions(file.type_char(), file.permissions(), xattrs),
            Column::FileSize(fmt)        => self.render_size(file.size(), fmt),
            Column::Timestamp(Modified)  => self.render_time(file.modified_time()),
            Column::Timestamp(Created)   => self.render_time(file.created_time()),
            Column::Timestamp(Accessed)  => self.render_time(file.accessed_time()),
            Column::HardLinks            => self.render_links(file.links()),
            Column::Inode                => self.render_inode(file.inode()),
            Column::Blocks               => self.render_blocks(file.blocks()),
            Column::User                 => self.render_user(file.user()),
            Column::Group                => self.render_group(file.group()),
            Column::GitStatus            => self.render_git_status(file.git_status()),
        }
    }

    fn render_permissions(&self, file_type: f::Type, permissions: f::Permissions, xattrs: bool) -> TextCell {
        let perms = self.opts.colours.perms;
        let types = self.opts.colours.filetypes;

        let bit = |bit, chr: &'static str, style: Style| {
            if bit { style.paint(chr) } else { self.opts.colours.punctuation.paint("-") }
        };

        let type_char = match file_type {
            f::Type::File       => types.normal.paint("."),
            f::Type::Directory  => types.directory.paint("d"),
            f::Type::Pipe       => types.pipe.paint("|"),
            f::Type::Link       => types.symlink.paint("l"),
            f::Type::CharDevice => types.device.paint("c"),
            f::Type::BlockDevice => types.device.paint("b"),
            f::Type::Socket     => types.socket.paint("s"),
            f::Type::Special    => types.special.paint("?"),
        };

        let x_colour = if file_type.is_regular_file() { perms.user_execute_file }
                                                 else { perms.user_execute_other };

        let mut chars = vec![
            type_char,
            bit(permissions.user_read,     "r", perms.user_read),
            bit(permissions.user_write,    "w", perms.user_write),
            bit(permissions.user_execute,  "x", x_colour),
            bit(permissions.group_read,    "r", perms.group_read),
            bit(permissions.group_write,   "w", perms.group_write),
            bit(permissions.group_execute, "x", perms.group_execute),
            bit(permissions.other_read,    "r", perms.other_read),
            bit(permissions.other_write,   "w", perms.other_write),
            bit(permissions.other_execute, "x", perms.other_execute),
        ];

        if xattrs {
            chars.push(perms.attribute.paint("@"));
        }

        // As these are all ASCII characters, we can guarantee that they’re
        // all going to be one character wide, and don’t need to compute the
        // cell’s display width.
        let width = DisplayWidth::from(chars.len());

        TextCell {
            contents: chars.into(),
            width:    width,
        }
    }

    fn render_links(&self, links: f::Links) -> TextCell {
        let style = if links.multiple { self.opts.colours.links.multi_link_file }
                                 else { self.opts.colours.links.normal };

        TextCell::paint(style, self.env.numeric.format_int(links.count))
    }

    fn render_blocks(&self, blocks: f::Blocks) -> TextCell {
        match blocks {
            f::Blocks::Some(blk)  => TextCell::paint(self.opts.colours.blocks, blk.to_string()),
            f::Blocks::None       => TextCell::blank(self.opts.colours.punctuation),
        }
    }

    fn render_inode(&self, inode: f::Inode) -> TextCell {
        TextCell::paint(self.opts.colours.inode, inode.0.to_string())
    }

    fn render_size(&self, size: f::Size, size_format: SizeFormat) -> TextCell {
        use number_prefix::{binary_prefix, decimal_prefix};
        use number_prefix::{Prefixed, Standalone, PrefixNames};

        let size = match size {
            f::Size::Some(s) => s,
            f::Size::None => return TextCell::blank(self.opts.colours.punctuation),
        };

        let result = match size_format {
            SizeFormat::DecimalBytes  => decimal_prefix(size as f64),
            SizeFormat::BinaryBytes   => binary_prefix(size as f64),
            SizeFormat::JustBytes     => {
                let string = self.env.numeric.format_int(size);
                return TextCell::paint(self.opts.colours.file_size(size), string);
            },
        };

        let (prefix, n) = match result {
            Standalone(b)  => return TextCell::paint(self.opts.colours.file_size(b as u64), b.to_string()),
            Prefixed(p, n) => (p, n)
        };

        let symbol = prefix.symbol();
        let number = if n < 10f64 { self.env.numeric.format_float(n, 1) }
                             else { self.env.numeric.format_int(n as isize) };

        // The numbers and symbols are guaranteed to be written in ASCII, so
        // we can skip the display width calculation.
        let width = DisplayWidth::from(number.len() + symbol.len());

        TextCell {
            width:    width,
            contents: vec![
                self.opts.colours.file_size(size).paint(number),
                self.opts.colours.size.unit.paint(symbol),
            ].into(),
        }
    }

    #[allow(trivial_numeric_casts)]
    fn render_time(&self, timestamp: f::Time) -> TextCell {
        // TODO(ogham): This method needs some serious de-duping!
        // zoned and local times have different types at the moment,
        // so it's tricky.

        if let Some(ref tz) = self.env.tz {
            let date = tz.to_zoned(LocalDateTime::at(timestamp.0 as i64));

            let datestamp = if date.year() == self.env.current_year {
                DATE_AND_TIME.format(&date, &self.env.time)
            }
            else {
                DATE_AND_YEAR.format(&date, &self.env.time)
            };

            TextCell::paint(self.opts.colours.date, datestamp)
        }
        else {
            let date = LocalDateTime::at(timestamp.0 as i64);

            let datestamp = if date.year() == self.env.current_year {
                DATE_AND_TIME.format(&date, &self.env.time)
            }
            else {
                DATE_AND_YEAR.format(&date, &self.env.time)
            };

            TextCell::paint(self.opts.colours.date, datestamp)
        }
    }

    fn render_git_status(&self, git: f::Git) -> TextCell {
        let git_char = |status| match status {
            f::GitStatus::NotModified  => self.opts.colours.punctuation.paint("-"),
            f::GitStatus::New          => self.opts.colours.git.new.paint("N"),
            f::GitStatus::Modified     => self.opts.colours.git.modified.paint("M"),
            f::GitStatus::Deleted      => self.opts.colours.git.deleted.paint("D"),
            f::GitStatus::Renamed      => self.opts.colours.git.renamed.paint("R"),
            f::GitStatus::TypeChange   => self.opts.colours.git.typechange.paint("T"),
        };

        TextCell {
            width: DisplayWidth::from(2),
            contents: vec![
                git_char(git.staged),
                git_char(git.unstaged)
            ].into(),
        }
    }

    fn render_user(&self, user: f::User) -> TextCell {
        let users = self.env.users.lock().unwrap();


        let user_name = match users.get_user_by_uid(user.0) {
            Some(user)  => user.name().to_owned(),
            None        => user.0.to_string(),
        };

        let style = if users.get_current_uid() == user.0 { self.opts.colours.users.user_you }
                                                    else { self.opts.colours.users.user_someone_else };
        TextCell::paint(style, user_name)
    }

    fn render_group(&self, group: f::Group) -> TextCell {
        use users::os::unix::GroupExt;

        let mut style = self.opts.colours.users.group_not_yours;

        let users = self.env.users.lock().unwrap();
        let group = match users.get_group_by_gid(group.0) {
            Some(g) => (*g).clone(),
            None    => return TextCell::paint(style, group.0.to_string()),
        };

        let current_uid = users.get_current_uid();
        if let Some(current_user) = users.get_user_by_uid(current_uid) {
            if current_user.primary_group_id() == group.gid()
            || group.members().contains(&current_user.name().to_owned()) {
                style = self.opts.colours.users.group_yours;
            }
        }

        TextCell::paint(style, group.name().to_owned())
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


lazy_static! {
    static ref DATE_AND_TIME: DateFormat<'static> =
        DateFormat::parse("{2>:D} {:M} {2>:h}:{02>:m}").unwrap();

    static ref DATE_AND_YEAR: DateFormat<'static> =
        DateFormat::parse("{2>:D} {:M} {5>:Y}").unwrap();
}


#[cfg(test)]
pub mod test {
    pub use super::{Table, Environment, Details};
    pub use std::sync::Mutex;

    pub use fs::{File, fields as f};
    pub use output::column::{Column, Columns};
    pub use output::cell::TextCell;

    pub use users::{User, Group, uid_t, gid_t};
    pub use users::mock::MockUsers;
    pub use users::os::unix::{UserExt, GroupExt};

    pub use ansi_term::Style;
    pub use ansi_term::Colour::*;

    impl Default for Environment<MockUsers> {
        fn default() -> Self {
            use locale;
            use users::mock::MockUsers;
            use std::sync::Mutex;

            Environment {
                current_year: 1234,
                numeric:      locale::Numeric::english(),
                time:         locale::Time::english(),
                tz:           None,
                users:        Mutex::new(MockUsers::with_current_uid(0)),
            }
        }
    }

    pub fn new_table<'a>(columns: &'a [Column], details: &'a Details, users: MockUsers) -> Table<'a, MockUsers> {
        use std::sync::Arc;

        Table {
            columns: columns,
            opts: details,
            env: Arc::new(Environment { users: Mutex::new(users), ..Environment::default() }),
            rows: Vec::new(),
        }
    }

    mod users {
        #![allow(unused_results)]
        use super::*;

        #[test]
        fn named() {
            let columns = Columns::default().for_dir(None);
            let mut details = Details::default();
            details.colours.users.user_you = Red.bold();

            let mut users = MockUsers::with_current_uid(1000);
            users.add_user(User::new(1000, "enoch", 100));

            let table = new_table(&columns, &details, users);

            let user = f::User(1000);
            let expected = TextCell::paint_str(Red.bold(), "enoch");
            assert_eq!(expected, table.render_user(user))
        }

        #[test]
        fn unnamed() {
            let columns = Columns::default().for_dir(None);
            let mut details = Details::default();
            details.colours.users.user_you = Cyan.bold();

            let users = MockUsers::with_current_uid(1000);

            let table = new_table(&columns, &details, users);

            let user = f::User(1000);
            let expected = TextCell::paint_str(Cyan.bold(), "1000");
            assert_eq!(expected, table.render_user(user));
        }

        #[test]
        fn different_named() {
            let columns = Columns::default().for_dir(None);
            let mut details = Details::default();
            details.colours.users.user_someone_else = Green.bold();

            let table = new_table(&columns, &details, MockUsers::with_current_uid(0));
            table.env.users.lock().unwrap().add_user(User::new(1000, "enoch", 100));

            let user = f::User(1000);
            let expected = TextCell::paint_str(Green.bold(), "enoch");
            assert_eq!(expected, table.render_user(user));
        }

        #[test]
        fn different_unnamed() {
            let columns = Columns::default().for_dir(None);
            let mut details = Details::default();
            details.colours.users.user_someone_else = Red.normal();

            let table = new_table(&columns, &details, MockUsers::with_current_uid(0));

            let user = f::User(1000);
            let expected = TextCell::paint_str(Red.normal(), "1000");
            assert_eq!(expected, table.render_user(user));
        }

        #[test]
        fn overflow() {
            let columns = Columns::default().for_dir(None);
            let mut details = Details::default();
            details.colours.users.user_someone_else = Blue.underline();

            let table = new_table(&columns, &details, MockUsers::with_current_uid(0));

            let user = f::User(2_147_483_648);
            let expected = TextCell::paint_str(Blue.underline(), "2147483648");
            assert_eq!(expected, table.render_user(user));
        }
    }

    mod groups {
        #![allow(unused_results)]
        use super::*;

        #[test]
        fn named() {
            let columns = Columns::default().for_dir(None);
            let mut details = Details::default();
            details.colours.users.group_not_yours = Fixed(101).normal();

            let mut users = MockUsers::with_current_uid(1000);
            users.add_group(Group::new(100, "folk"));
            let table = new_table(&columns, &details, users);

            let group = f::Group(100);
            let expected = TextCell::paint_str(Fixed(101).normal(), "folk");
            assert_eq!(expected, table.render_group(group))
        }

        #[test]
        fn unnamed() {
            let columns = Columns::default().for_dir(None);
            let mut details = Details::default();
            details.colours.users.group_not_yours = Fixed(87).normal();

            let users = MockUsers::with_current_uid(1000);
            let table = new_table(&columns, &details, users);

            let group = f::Group(100);
            let expected = TextCell::paint_str(Fixed(87).normal(), "100");
            assert_eq!(expected, table.render_group(group));
        }

        #[test]
        fn primary() {
            let columns = Columns::default().for_dir(None);
            let mut details = Details::default();
            details.colours.users.group_yours = Fixed(64).normal();

            let mut users = MockUsers::with_current_uid(2);
            users.add_user(User::new(2, "eve", 100));
            users.add_group(Group::new(100, "folk"));

            let table = new_table(&columns, &details, users);

            let group = f::Group(100);
            let expected = TextCell::paint_str(Fixed(64).normal(), "folk");
            assert_eq!(expected, table.render_group(group))
        }

        #[test]
        fn secondary() {
            let columns = Columns::default().for_dir(None);
            let mut details = Details::default();
            details.colours.users.group_yours = Fixed(31).normal();

            let mut users = MockUsers::with_current_uid(2);
            users.add_user(User::new(2, "eve", 666));

            let test_group = Group::new(100, "folk").add_member("eve");
            users.add_group(test_group);

            let table = new_table(&columns, &details, users);

            let group = f::Group(100);
            let expected = TextCell::paint_str(Fixed(31).normal(), "folk");
            assert_eq!(expected, table.render_group(group))
        }

        #[test]
        fn overflow() {
            let columns = Columns::default().for_dir(None);
            let mut details = Details::default();
            details.colours.users.group_not_yours = Blue.underline();

            let table = new_table(&columns, &details, MockUsers::with_current_uid(0));

            let group = f::Group(2_147_483_648);
            let expected = TextCell::paint_str(Blue.underline(), "2147483648");
            assert_eq!(expected, table.render_group(group));
        }
    }
}
