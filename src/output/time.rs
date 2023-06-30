//! Timestamp formatting.

use std::time::{SystemTime, UNIX_EPOCH};

use datetime::{LocalDateTime, TimeZone, DatePiece, TimePiece};
use datetime::fmt::DateFormat;

use lazy_static::lazy_static;
use unicode_width::UnicodeWidthStr;


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
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum TimeFormat {

    /// The **default format** uses the user’s locale to print month names,
    /// and specifies the timestamp down to the minute for recent times, and
    /// day for older times.
    DefaultFormat,

    /// Use the **ISO format**, which specifies the timestamp down to the
    /// minute for recent times, and day for older times. It uses a number
    /// for the month so it doesn’t use the locale.
    ISOFormat,

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
    pub fn format_local(self, time: SystemTime) -> String {
        match self {
            Self::DefaultFormat  => default_local(time),
            Self::ISOFormat      => iso_local(time),
            Self::LongISO        => long_local(time),
            Self::FullISO        => full_local(time),
        }
    }

    pub fn format_zoned(self, time: SystemTime, zone: &TimeZone) -> String {
        match self {
            Self::DefaultFormat  => default_zoned(time, zone),
            Self::ISOFormat      => iso_zoned(time, zone),
            Self::LongISO        => long_zoned(time, zone),
            Self::FullISO        => full_zoned(time, zone),
        }
    }
}


#[allow(trivial_numeric_casts)]
fn default_local(time: SystemTime) -> String {
    let date = LocalDateTime::at(systemtime_epoch(time));
    let date_format = get_dateformat(&date);
    date_format.format(&date, &*LOCALE)
}

#[allow(trivial_numeric_casts)]
fn default_zoned(time: SystemTime, zone: &TimeZone) -> String {
    let date = zone.to_zoned(LocalDateTime::at(systemtime_epoch(time)));
    let date_format = get_dateformat(&date);
    date_format.format(&date, &*LOCALE)
}

fn get_dateformat(date: &LocalDateTime) -> &'static DateFormat<'static> {
    match (is_recent(date), *MAXIMUM_MONTH_WIDTH) {
        (true, 4)   => &FOUR_WIDE_DATE_TIME,
        (true, 5)   => &FIVE_WIDE_DATE_TIME,
        (true, _)   => &OTHER_WIDE_DATE_TIME,
        (false, 4)  => &FOUR_WIDE_DATE_YEAR,
        (false, 5)  => &FIVE_WIDE_DATE_YEAR,
        (false, _)  => &OTHER_WIDE_DATE_YEAR,
    }
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

#[allow(trivial_numeric_casts)]
fn iso_local(time: SystemTime) -> String {
    let date = LocalDateTime::at(systemtime_epoch(time));

    if is_recent(&date) {
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
fn iso_zoned(time: SystemTime, zone: &TimeZone) -> String {
    let date = zone.to_zoned(LocalDateTime::at(systemtime_epoch(time)));

    if is_recent(&date) {
        format!("{:02}-{:02} {:02}:{:02}",
                date.month() as usize, date.day(),
                date.hour(), date.minute())
    }
    else {
        format!("{:04}-{:02}-{:02}",
                date.year(), date.month() as usize, date.day())
    }
}


fn systemtime_epoch(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH)
        .map(|t| t.as_secs() as i64)
        .unwrap_or_else(|e| {
            let diff = e.duration();
            let mut secs = diff.as_secs();
            if diff.subsec_nanos() > 0 {
                secs += 1;
            }
            -(secs as i64)
        })
}

fn systemtime_nanos(time: SystemTime) -> u32 {
    time.duration_since(UNIX_EPOCH)
        .map(|t| t.subsec_nanos())
        .unwrap_or_else(|e| {
            let nanos = e.duration().subsec_nanos();
            if nanos > 0 {
                1_000_000_000 - nanos
            } else {
                nanos
            }
        })
}

fn is_recent(date: &LocalDateTime) -> bool {
    date.year() == *CURRENT_YEAR
}


lazy_static! {

    static ref CURRENT_YEAR: i64 = LocalDateTime::now().year();

    static ref LOCALE: locale::Time = {
        locale::Time::load_user_locale()
               .unwrap_or_else(|_| locale::Time::english())
    };

    static ref MAXIMUM_MONTH_WIDTH: usize = {
        // Some locales use a three-character wide month name (Jan to Dec);
        // others vary between three to four (1月 to 12月, juil.). We check each month width
        // to detect the longest and set the output format accordingly.
        let mut maximum_month_width = 0;
        for i in 0..11 {
            let current_month_width = UnicodeWidthStr::width(&*LOCALE.short_month_name(i));
            maximum_month_width = std::cmp::max(maximum_month_width, current_month_width);
        }
        maximum_month_width
    };

    static ref FOUR_WIDE_DATE_TIME: DateFormat<'static> = DateFormat::parse(
        "{2>:D} {4<:M} {02>:h}:{02>:m}"
    ).unwrap();

    static ref FIVE_WIDE_DATE_TIME: DateFormat<'static> = DateFormat::parse(
        "{2>:D} {5<:M} {02>:h}:{02>:m}"
    ).unwrap();

    static ref OTHER_WIDE_DATE_TIME: DateFormat<'static> = DateFormat::parse(
        "{2>:D} {:M} {02>:h}:{02>:m}"
    ).unwrap();

    static ref FOUR_WIDE_DATE_YEAR: DateFormat<'static> = DateFormat::parse(
        "{2>:D} {4<:M} {5>:Y}"
    ).unwrap();

    static ref FIVE_WIDE_DATE_YEAR: DateFormat<'static> = DateFormat::parse(
        "{2>:D} {5<:M} {5>:Y}"
    ).unwrap();

    static ref OTHER_WIDE_DATE_YEAR: DateFormat<'static> = DateFormat::parse(
        "{2>:D} {:M} {5>:Y}"
    ).unwrap();
}
