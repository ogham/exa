//! Timestamp formatting.

use std::time::{SystemTime, UNIX_EPOCH};

use datetime::{LocalDateTime, TimeZone, DatePiece, TimePiece};
use datetime::fmt::DateFormat;
use locale;
use std::cmp;


/// Every timestamp in exa needs to be rendered by a **time format**.
/// Formatting times is tricky, because how a timestamp is rendered can
/// depend on one or more of the following:
///
/// - The user’s locale, for printing the month name as “Feb”, or as “fév”,
///   or as “2月”;
/// - The current year, because certain formats will be less precise when
///   dealing with dates far in the past;
/// - The formatting style that the user asked for on the command-line.
///
/// Because not all formatting styles need the same data, they all have their
/// own enum variants. It’s not worth looking the locale up if the formatter
/// prints month names as numbers.
///
/// Currently exa does not support *custom* styles, where the user enters a
/// format string in an environment variable or something. Just these four.
#[derive(Debug)]
pub enum TimeFormat {

    /// The **default format** uses the user’s locale to print month names,
    /// and specifies the timestamp down to the minute for recent times, and
    /// day for older times.
    DefaultFormat(DefaultFormat),

    /// Use the **ISO format**, which specifies the timestamp down to the
    /// minute for recent times, and day for older times. It uses a number
    /// for the month so it doesn’t need a locale.
    ISOFormat(ISOFormat),

    /// Use the **long ISO format**, which specifies the timestamp down to the
    /// minute using only numbers, without needing the locale or year.
    LongISO,

    /// Use the **full ISO format**, which specifies the timestamp down to the
    /// millisecond and includes its offset down to the minute. This too uses
    /// only numbers so doesn’t require any special consideration.
    FullISO,
}

// There are two different formatting functions because local and zoned
// timestamps are separate types.

impl TimeFormat {
    pub fn format_local(&self, time: SystemTime) -> String {
        match *self {
            TimeFormat::DefaultFormat(ref fmt) => fmt.format_local(time),
            TimeFormat::ISOFormat(ref iso)     => iso.format_local(time),
            TimeFormat::LongISO                => long_local(time),
            TimeFormat::FullISO                => full_local(time),
        }
    }

    pub fn format_zoned(&self, time: SystemTime, zone: &TimeZone) -> String {
        match *self {
            TimeFormat::DefaultFormat(ref fmt) => fmt.format_zoned(time, zone),
            TimeFormat::ISOFormat(ref iso)     => iso.format_zoned(time, zone),
            TimeFormat::LongISO                => long_zoned(time, zone),
            TimeFormat::FullISO                => full_zoned(time, zone),
        }
    }
}


#[derive(Debug, Clone)]
pub struct DefaultFormat {

    /// The year of the current time. This gets used to determine which date
    /// format to use.
    pub current_year: i64,

    /// Localisation rules for formatting timestamps.
    pub locale: locale::Time,

    /// Date format for printing out timestamps that are in the current year.
    pub date_and_time: DateFormat<'static>,

    /// Date format for printing out timestamps that *aren’t*.
    pub date_and_year: DateFormat<'static>,
}

impl DefaultFormat {
    pub fn load() -> DefaultFormat {
        use unicode_width::UnicodeWidthStr;

        let locale = locale::Time::load_user_locale()
                       .unwrap_or_else(|_| locale::Time::english());

        let current_year = LocalDateTime::now().year();

        // Some locales use a three-character wide month name (Jan to Dec);
        // others vary between three to four (1月 to 12月, juil.). We check each month width
        // to detect the longest and set the output format accordingly.
        let mut maximum_month_width = 0;
        for i in 0..11 {
            let current_month_width = UnicodeWidthStr::width(&*locale.short_month_name(i));
            maximum_month_width = cmp::max(maximum_month_width, current_month_width);
        }

        let date_and_time = match maximum_month_width {
            4  => DateFormat::parse("{2>:D} {4<:M} {2>:h}:{02>:m}").unwrap(),
            5  => DateFormat::parse("{2>:D} {5<:M} {2>:h}:{02>:m}").unwrap(),
            _  => DateFormat::parse("{2>:D} {:M} {2>:h}:{02>:m}").unwrap(),
        };

        let date_and_year = match maximum_month_width {
            4 => DateFormat::parse("{2>:D} {4<:M} {5>:Y}").unwrap(),
            5 => DateFormat::parse("{2>:D} {5<:M} {5>:Y}").unwrap(),
            _ => DateFormat::parse("{2>:D} {:M} {5>:Y}").unwrap()
        };

        DefaultFormat { current_year, locale, date_and_time, date_and_year }
    }
}


impl DefaultFormat {
    fn is_recent(&self, date: LocalDateTime) -> bool {
        date.year() == self.current_year
    }

    fn month_to_abbrev(month: datetime::Month) -> &'static str {
        match month {
            datetime::Month::January => "Jan",
            datetime::Month::February => "Feb",
            datetime::Month::March => "Mar",
            datetime::Month::April => "Apr",
            datetime::Month::May => "May",
            datetime::Month::June => "Jun",
            datetime::Month::July => "Jul",
            datetime::Month::August => "August",
            datetime::Month::September => "Sep",
            datetime::Month::October => "Oct",
            datetime::Month::November => "Nov",
            datetime::Month::December => "Dec",
        }
    }

    #[allow(trivial_numeric_casts)]
    fn format_local(&self, time: SystemTime) -> String {
        if time == UNIX_EPOCH {
            return "-".to_string();
        }
        let date = LocalDateTime::at(systemtime_epoch(time));

        if self.is_recent(date) {
            format!("{:2} {} {:02}:{:02}",
            date.day(), DefaultFormat::month_to_abbrev(date.month()),
            date.hour(), date.minute())
        }
        else {
            self.date_and_year.format(&date, &self.locale)
        }
    }

    #[allow(trivial_numeric_casts)]
    fn format_zoned(&self, time: SystemTime, zone: &TimeZone) -> String {
        if time == UNIX_EPOCH {
            return "-".to_string();
        }

        let date = zone.to_zoned(LocalDateTime::at(systemtime_epoch(time)));

        if self.is_recent(date) {
            format!("{:2} {} {:02}:{:02}",
            date.day(), DefaultFormat::month_to_abbrev(date.month()),
            date.hour(), date.minute())
        }
        else {
            self.date_and_year.format(&date, &self.locale)
        }
    }
}

fn systemtime_epoch(time: SystemTime) -> i64 {
    time
        .duration_since(UNIX_EPOCH)
        .map(|t| t.as_secs() as i64)
        .unwrap_or_else(|e| -(e.duration().as_secs() as i64))
}

fn systemtime_nanos(time: SystemTime) -> u32 {
    time
        .duration_since(UNIX_EPOCH)
        .map(|t| t.subsec_nanos())
        .unwrap_or_else(|e| e.duration().subsec_nanos())
}

#[allow(trivial_numeric_casts)]
fn long_local(time: SystemTime) -> String {
    let date = LocalDateTime::at(systemtime_epoch(time));
    format!("{:04}-{:02}-{:02} {:02}:{:02}",
            date.year(), date.month() as usize, date.day(),
            date.hour(), date.minute())
}

#[allow(trivial_numeric_casts)]
fn long_zoned(time: SystemTime, zone: &TimeZone) -> String {
    let date = zone.to_zoned(LocalDateTime::at(systemtime_epoch(time)));
    format!("{:04}-{:02}-{:02} {:02}:{:02}",
            date.year(), date.month() as usize, date.day(),
            date.hour(), date.minute())
}


#[allow(trivial_numeric_casts)]
fn full_local(time: SystemTime) -> String {
    let date = LocalDateTime::at(systemtime_epoch(time));
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:09}",
            date.year(), date.month() as usize, date.day(),
            date.hour(), date.minute(), date.second(), systemtime_nanos(time))
}

#[allow(trivial_numeric_casts)]
fn full_zoned(time: SystemTime, zone: &TimeZone) -> String {
    use datetime::Offset;

    let local = LocalDateTime::at(systemtime_epoch(time));
    let date = zone.to_zoned(local);
    let offset = Offset::of_seconds(zone.offset(local) as i32).expect("Offset out of range");
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:09} {:+03}{:02}",
            date.year(), date.month() as usize, date.day(),
            date.hour(), date.minute(), date.second(), systemtime_nanos(time),
            offset.hours(), offset.minutes().abs())
}



#[derive(Debug, Clone)]
pub struct ISOFormat {

    /// The year of the current time. This gets used to determine which date
    /// format to use.
    pub current_year: i64,
}

impl ISOFormat {
    pub fn load() -> ISOFormat {
        let current_year = LocalDateTime::now().year();
        ISOFormat { current_year }
    }
}

impl ISOFormat {
    fn is_recent(&self, date: LocalDateTime) -> bool {
        date.year() == self.current_year
    }

    #[allow(trivial_numeric_casts)]
    fn format_local(&self, time: SystemTime) -> String {
        let date = LocalDateTime::at(systemtime_epoch(time));

        if self.is_recent(date) {
            format!("{:02}-{:02} {:02}:{:02}",
                    date.month() as usize, date.day(),
                    date.hour(), date.minute())
        }
        else {
            format!("{:04}-{:02}-{:02}",
                    date.year(), date.month() as usize, date.day())
        }
    }

    #[allow(trivial_numeric_casts)]
    fn format_zoned(&self, time: SystemTime, zone: &TimeZone) -> String {
        let date = zone.to_zoned(LocalDateTime::at(systemtime_epoch(time)));

        if self.is_recent(date) {
            format!("{:02}-{:02} {:02}:{:02}",
                    date.month() as usize, date.day(),
                    date.hour(), date.minute())
        }
        else {
            format!("{:04}-{:02}-{:02}",
                    date.year(), date.month() as usize, date.day())
        }
    }
}
