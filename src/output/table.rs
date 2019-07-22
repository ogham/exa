use std::cmp::max;
use std::fmt;
use std::ops::Deref;
use std::sync::{Mutex, MutexGuard};

use datetime::TimeZone;
use zoneinfo_compiled::{CompiledData, Result as TZResult};

use locale;

use users::UsersCache;

use style::Colours;
use output::cell::TextCell;
use output::render::TimeRender;
use output::time::TimeFormat;
use fs::{File, fields as f};
use fs::feature::git::GitCache;


/// Options for displaying a table.
pub struct Options {
    pub env: Environment,
    pub size_format: SizeFormat,
    pub time_format: TimeFormat,
    pub extra_columns: Columns,
}

// I had to make other types derive Debug,
// and Mutex<UsersCache> is not that!
impl fmt::Debug for Options {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Table({:#?})", self.extra_columns)
    }
}

/// Extra columns to display in the table.
#[derive(PartialEq, Debug)]
pub struct Columns {

    /// At least one of these timestamps will be shown.
    pub time_types: TimeTypes,

    // The rest are just on/off
    pub inode: bool,
    pub links: bool,
    pub blocks: bool,
    pub group: bool,
    pub git: bool,
}

impl Columns {
    pub fn collect(&self, actually_enable_git: bool) -> Vec<Column> {
        let mut columns = Vec::with_capacity(4);

        if self.inode {
            columns.push(Column::Inode);
        }

        columns.push(Column::Permissions);

        if self.links {
            columns.push(Column::HardLinks);
        }

        columns.push(Column::FileSize);

        if self.blocks {
            columns.push(Column::Blocks);
        }

        columns.push(Column::User);

        if self.group {
            columns.push(Column::Group);
        }

        if self.time_types.modified {
            columns.push(Column::Timestamp(TimeType::Modified));
        }

        if self.time_types.changed {
            columns.push(Column::Timestamp(TimeType::Changed));
        }

        if self.time_types.created {
            columns.push(Column::Timestamp(TimeType::Created));
        }

        if self.time_types.accessed {
            columns.push(Column::Timestamp(TimeType::Accessed));
        }

        if cfg!(feature="git") && self.git && actually_enable_git {
            columns.push(Column::GitStatus);
        }

        columns
    }
}


/// A table contains these.
#[derive(Debug)]
pub enum Column {
    Permissions,
    FileSize,
    Timestamp(TimeType),
    Blocks,
    User,
    Group,
    HardLinks,
    Inode,
    GitStatus,
}

/// Each column can pick its own **Alignment**. Usually, numbers are
/// right-aligned, and text is left-aligned.
#[derive(Copy, Clone)]
pub enum Alignment {
    Left, Right,
}

impl Column {

    /// Get the alignment this column should use.
    pub fn alignment(&self) -> Alignment {
        match *self {
            Column::FileSize
            | Column::HardLinks
            | Column::Inode
            | Column::Blocks
            | Column::GitStatus => Alignment::Right,
            _                   => Alignment::Left,
        }
    }

    /// Get the text that should be printed at the top, when the user elects
    /// to have a header row printed.
    pub fn header(&self) -> &'static str {
        match *self {
            Column::Permissions   => "Permissions",
            Column::FileSize      => "Size",
            Column::Timestamp(t)  => t.header(),
            Column::Blocks        => "Blocks",
            Column::User          => "User",
            Column::Group         => "Group",
            Column::HardLinks     => "Links",
            Column::Inode         => "inode",
            Column::GitStatus     => "Git",
        }
    }
}


/// Formatting options for file sizes.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum SizeFormat {

    /// Format the file size using **decimal** prefixes, such as “kilo”,
    /// “mega”, or “giga”.
    DecimalBytes,

    /// Format the file size using **binary** prefixes, such as “kibi”,
    /// “mebi”, or “gibi”.
    BinaryBytes,

    /// Do no formatting and just display the size as a number of bytes.
    JustBytes,
}

impl Default for SizeFormat {
    fn default() -> SizeFormat {
        SizeFormat::DecimalBytes
    }
}


/// The types of a file’s time fields. These three fields are standard
/// across most (all?) operating systems.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TimeType {
    /// The file’s modified time (`st_mtime`).
    Modified,

    /// The file’s changed time (`st_ctime`)
    Changed,

    /// The file’s accessed time (`st_atime`).
    Accessed,

    /// The file’s creation time (`btime` or `birthtime`).
    Created,
}

impl TimeType {

    /// Returns the text to use for a column’s heading in the columns output.
    pub fn header(self) -> &'static str {
        match self {
            TimeType::Modified  => "Date Modified",
            TimeType::Changed   => "Date Changed",
            TimeType::Accessed  => "Date Accessed",
            TimeType::Created   => "Date Created",
        }
    }
}


/// Fields for which of a file’s time fields should be displayed in the
/// columns output.
///
/// There should always be at least one of these--there's no way to disable
/// the time columns entirely (yet).
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct TimeTypes {
    pub modified: bool,
    pub changed:  bool,
    pub accessed: bool,
    pub created:  bool,
}

impl Default for TimeTypes {

    /// By default, display just the ‘modified’ time. This is the most
    /// common option, which is why it has this shorthand.
    fn default() -> TimeTypes {
        TimeTypes { modified: true, changed: false, accessed: false, created: false }
    }
}




/// The **environment** struct contains any data that could change between
/// running instances of exa, depending on the user's computer's configuration.
///
/// Any environment field should be able to be mocked up for test runs.
pub struct Environment {

    /// Localisation rules for formatting numbers.
    numeric: locale::Numeric,

    /// The computer's current time zone. This gets used to determine how to
    /// offset files' timestamps.
    tz: Option<TimeZone>,

    /// Mapping cache of user IDs to usernames.
    users: Mutex<UsersCache>,
}

impl Environment {
    pub fn lock_users(&self) -> MutexGuard<UsersCache> {
        self.users.lock().unwrap()
    }

    pub fn load_all() -> Self {
        let tz = match determine_time_zone() {
            Ok(t) => Some(t),
            Err(ref e) => {
                println!("Unable to determine time zone: {}", e);
                None
            }
        };

        let numeric = locale::Numeric::load_user_locale()
                          .unwrap_or_else(|_| locale::Numeric::english());

        let users = Mutex::new(UsersCache::new());

        Environment { tz, numeric, users }
    }
}

fn determine_time_zone() -> TZResult<TimeZone> {
    TimeZone::from_file("/etc/localtime")
}


pub struct Table<'a> {
    columns: Vec<Column>,
    colours: &'a Colours,
    env: &'a Environment,
    widths: TableWidths,
    time_format: &'a TimeFormat,
    size_format: SizeFormat,
    git: Option<&'a GitCache>,
}

#[derive(Clone)]
pub struct Row {
    cells: Vec<TextCell>,
}

impl<'a, 'f> Table<'a> {
    pub fn new(options: &'a Options, git: Option<&'a GitCache>, colours: &'a Colours) -> Table<'a> {
        let columns = options.extra_columns.collect(git.is_some());
        let widths = TableWidths::zero(columns.len());

        Table {
            colours, widths, columns, git,
            env:         &options.env,
            time_format: &options.time_format,
            size_format:  options.size_format,
        }
    }

    pub fn widths(&self) -> &TableWidths {
        &self.widths
    }

    pub fn header_row(&self) -> Row {
        let cells = self.columns.iter()
                        .map(|c| TextCell::paint_str(self.colours.header, c.header()))
                        .collect();

        Row { cells }
    }

    pub fn row_for_file(&self, file: &File, xattrs: bool) -> Row {
        let cells = self.columns.iter()
                        .map(|c| self.display(file, c, xattrs))
                        .collect();

        Row { cells }
    }

    pub fn add_widths(&mut self, row: &Row) {
        self.widths.add_widths(row)
    }

    fn permissions_plus(&self, file: &File, xattrs: bool) -> f::PermissionsPlus {
        f::PermissionsPlus {
            file_type: file.type_char(),
            permissions: file.permissions(),
            xattrs,
        }
    }

    fn display(&self, file: &File, column: &Column, xattrs: bool) -> TextCell {
        use output::table::TimeType::*;

        match *column {
            Column::Permissions    => self.permissions_plus(file, xattrs).render(self.colours),
            Column::FileSize       => file.size().render(self.colours, self.size_format, &self.env.numeric),
            Column::HardLinks      => file.links().render(self.colours, &self.env.numeric),
            Column::Inode          => file.inode().render(self.colours.inode),
            Column::Blocks         => file.blocks().render(self.colours),
            Column::User           => file.user().render(self.colours, &*self.env.lock_users()),
            Column::Group          => file.group().render(self.colours, &*self.env.lock_users()),
            Column::GitStatus      => self.git_status(file).render(self.colours),

            Column::Timestamp(Modified)  => file.modified_time().render(self.colours.date, &self.env.tz, &self.time_format),
            Column::Timestamp(Changed)   => file.changed_time() .render(self.colours.date, &self.env.tz, &self.time_format),
            Column::Timestamp(Created)   => file.created_time() .render(self.colours.date, &self.env.tz, &self.time_format),
            Column::Timestamp(Accessed)  => file.accessed_time().render(self.colours.date, &self.env.tz, &self.time_format),
        }
    }

    fn git_status(&self, file: &File) -> f::Git {
        debug!("Getting Git status for file {:?}", file.path);
        self.git
            .map(|g| g.get(&file.path, file.is_directory()))
            .unwrap_or_default()
    }

    pub fn render(&self, row: Row) -> TextCell {
        let mut cell = TextCell::default();

        for (n, (this_cell, width)) in row.cells.into_iter().zip(self.widths.iter()).enumerate() {
            let padding = width - *this_cell.width;

            match self.columns[n].alignment() {
                Alignment::Left  => { cell.append(this_cell); cell.add_spaces(padding); }
                Alignment::Right => { cell.add_spaces(padding); cell.append(this_cell); }
            }

            cell.add_spaces(1);
        }

        cell
    }
}



pub struct TableWidths(Vec<usize>);

impl Deref for TableWidths {
    type Target = [usize];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TableWidths {
    pub fn zero(count: usize) -> TableWidths {
        TableWidths(vec![ 0; count ])
    }

    pub fn add_widths(&mut self, row: &Row) {
        for (old_width, cell) in self.0.iter_mut().zip(row.cells.iter()) {
            *old_width = max(*old_width, *cell.width);
        }
    }

    pub fn total(&self) -> usize {
        self.0.len() + self.0.iter().sum::<usize>()
    }
}
