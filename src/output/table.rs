use std::cmp::max;
use std::sync::{Mutex, MutexGuard};

use datetime::fmt::DateFormat;
use datetime::{LocalDateTime, DatePiece};
use datetime::TimeZone;
use zoneinfo_compiled::{CompiledData, Result as TZResult};

use locale;

use users::UsersCache;

use output::cell::TextCell;
use output::colours::Colours;
use output::column::{Alignment, Column};

use fs::{File, fields as f};


/// The **environment** struct contains any data that could change between
/// running instances of exa, depending on the user's computer's configuration.
///
/// Any environment field should be able to be mocked up for test runs.
pub struct Environment {

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
    users: Mutex<UsersCache>,
}

impl Environment {
    pub fn lock_users(&self) -> MutexGuard<UsersCache> {
        self.users.lock().unwrap()
    }
}

impl Default for Environment {
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





pub struct Table<'a> {
    columns: &'a [Column],
    colours: &'a Colours,
    env: &'a Environment,
    widths: Vec<usize>,
}

#[derive(Clone)]
pub struct Row {
    cells: Vec<TextCell>,
}

impl<'a, 'f> Table<'a> {
    pub fn new(columns: &'a [Column], colours: &'a Colours, env: &'a Environment) -> Table<'a> {
        let widths = vec![ 0; columns.len() ];
        Table { columns, colours, env, widths }
    }

    pub fn columns_count(&self) -> usize {
        self.columns.len()
    }

    pub fn widths(&self) -> &[usize] {
        &self.widths
    }

    pub fn header_row(&mut self) -> Row {
        let mut cells = Vec::with_capacity(self.columns.len());

        for (old_width, column) in self.widths.iter_mut().zip(self.columns.iter()) {
            let column = TextCell::paint_str(self.colours.header, column.header());
            *old_width = max(*old_width, *column.width);
            cells.push(column);
        }

        Row { cells }
    }

    pub fn row_for_file(&mut self, file: &File, xattrs: bool) -> Row {
        let mut cells = Vec::with_capacity(self.columns.len());

        let other = self.columns.iter().map(|c| self.display(file, c, xattrs)).collect::<Vec<_>>();
        for (old_width, column) in self.widths.iter_mut().zip(other.into_iter()) {
            *old_width = max(*old_width, *column.width);
            cells.push(column);
        }

        Row { cells }
    }

    pub fn add_widths(&mut self, row: &Row) {
        for (old_width, cell) in self.widths.iter_mut().zip(row.cells.iter()) {
            *old_width = max(*old_width, *cell.width);
        }
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
            Column::Permissions          => self.permissions_plus(file, xattrs).render(&self.colours),
            Column::FileSize(fmt)        => file.size().render(&self.colours, fmt, &self.env.numeric),
            Column::Timestamp(Modified)  => file.modified_time().render(&self.colours, &self.env.tz, &self.env.date_and_time, &self.env.date_and_year, &self.env.time, self.env.current_year),
            Column::Timestamp(Created)   => file.created_time().render( &self.colours, &self.env.tz, &self.env.date_and_time, &self.env.date_and_year, &self.env.time, self.env.current_year),
            Column::Timestamp(Accessed)  => file.accessed_time().render(&self.colours, &self.env.tz, &self.env.date_and_time, &self.env.date_and_year, &self.env.time, self.env.current_year),
            Column::HardLinks            => file.links().render(&self.colours, &self.env.numeric),
            Column::Inode                => file.inode().render(&self.colours),
            Column::Blocks               => file.blocks().render(&self.colours),
            Column::User                 => file.user().render(&self.colours, &*self.env.lock_users()),
            Column::Group                => file.group().render(&self.colours, &*self.env.lock_users()),
            Column::GitStatus            => file.git_status().render(&self.colours),
        }
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
