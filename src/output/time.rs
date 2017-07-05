use datetime::{LocalDateTime, TimeZone, DatePiece, TimePiece};
use datetime::fmt::DateFormat;
use locale;

use fs::fields::Time;


pub enum TimeFormat {
    DefaultFormat(DefaultFormat),
    ISOFormat(ISOFormat),
    LongISO,
    FullISO,
}

impl TimeFormat {
    pub fn format_local(&self, time: Time) -> String {
        match *self {
            TimeFormat::DefaultFormat(ref fmt) => fmt.format_local(time),
            TimeFormat::ISOFormat(ref iso)     => iso.format_local(time),
            TimeFormat::LongISO                => long_local(time),
            TimeFormat::FullISO                => full_local(time),
        }
    }

    pub fn format_zoned(&self, time: Time, zone: &TimeZone) -> String {
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
    pub fn new() -> DefaultFormat {
        use unicode_width::UnicodeWidthStr;

        let locale = locale::Time::load_user_locale()
                       .unwrap_or_else(|_| locale::Time::english());

        let current_year = LocalDateTime::now().year();

        // Some locales use a three-character wide month name (Jan to Dec);
        // others vary between three and four (1月 to 12月). We assume that
        // December is the month with the maximum width, and use the width of
        // that to determine how to pad the other months.
        let december_width = UnicodeWidthStr::width(&*locale.short_month_name(11));
        let date_and_time = match december_width {
            4  => DateFormat::parse("{2>:D} {4>:M} {2>:h}:{02>:m}").unwrap(),
            _  => DateFormat::parse("{2>:D} {:M} {2>:h}:{02>:m}").unwrap(),
        };

        let date_and_year = match december_width {
            4 => DateFormat::parse("{2>:D} {4>:M} {5>:Y}").unwrap(),
            _ => DateFormat::parse("{2>:D} {:M} {5>:Y}").unwrap()
        };

        DefaultFormat { current_year, locale, date_and_time, date_and_year }
    }

    fn is_recent(&self, date: LocalDateTime) -> bool {
        date.year() == self.current_year
    }

    #[allow(trivial_numeric_casts)]
    fn format_local(&self, time: Time) -> String {
        let date = LocalDateTime::at(time.seconds as i64);

        if self.is_recent(date) {
            self.date_and_time.format(&date, &self.locale)
        }
        else {
            self.date_and_year.format(&date, &self.locale)
        }
    }

    #[allow(trivial_numeric_casts)]
    fn format_zoned(&self, time: Time, zone: &TimeZone) -> String {
        let date = zone.to_zoned(LocalDateTime::at(time.seconds as i64));

        if self.is_recent(date) {
            self.date_and_time.format(&date, &self.locale)
        }
        else {
            self.date_and_year.format(&date, &self.locale)
        }
    }
}


#[allow(trivial_numeric_casts)]
fn long_local(time: Time) -> String {
    let date = LocalDateTime::at(time.seconds as i64);
    format!("{:04}-{:02}-{:02} {:02}:{:02}",
            date.year(), date.month() as usize, date.day(),
            date.hour(), date.minute())
}

#[allow(trivial_numeric_casts)]
fn long_zoned(time: Time, zone: &TimeZone) -> String {
    let date = zone.to_zoned(LocalDateTime::at(time.seconds as i64));
    format!("{:04}-{:02}-{:02} {:02}:{:02}",
            date.year(), date.month() as usize, date.day(),
            date.hour(), date.minute())
}


#[allow(trivial_numeric_casts)]
fn full_local(time: Time) -> String {
    let date = LocalDateTime::at(time.seconds as i64);
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:09}",
            date.year(), date.month() as usize, date.day(),
            date.hour(), date.minute(), date.second(), time.nanoseconds)
}

#[allow(trivial_numeric_casts)]
fn full_zoned(time: Time, zone: &TimeZone) -> String {
    use datetime::Offset;

    let local = LocalDateTime::at(time.seconds as i64);
    let date = zone.to_zoned(local);
    let offset = Offset::of_seconds(zone.offset(local) as i32).expect("Offset out of range");
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:09} {:+03}{:02}",
            date.year(), date.month() as usize, date.day(),
            date.hour(), date.minute(), date.second(), time.nanoseconds,
            offset.hours(), offset.minutes().abs())
}



#[derive(Debug, Clone)]
pub struct ISOFormat {

    /// The year of the current time. This gets used to determine which date
    /// format to use.
    pub current_year: i64,
}

impl ISOFormat {
    pub fn new() -> Self {
        let current_year = LocalDateTime::now().year();
        ISOFormat { current_year }
    }

    fn is_recent(&self, date: LocalDateTime) -> bool {
        date.year() == self.current_year
    }

    #[allow(trivial_numeric_casts)]
    fn format_local(&self, time: Time) -> String {
        let date = LocalDateTime::at(time.seconds as i64);

        if self.is_recent(date) {
            format!("{:04}-{:02}-{:02}",
                    date.year(), date.month() as usize, date.day())
        }
        else {
            format!("{:02}-{:02} {:02}:{:02}",
                    date.month() as usize, date.day(),
                    date.hour(), date.minute())
        }
    }

    #[allow(trivial_numeric_casts)]
    fn format_zoned(&self, time: Time, zone: &TimeZone) -> String {
        let date = zone.to_zoned(LocalDateTime::at(time.seconds as i64));

        if self.is_recent(date) {
            format!("{:04}-{:02}-{:02}",
                    date.year(), date.month() as usize, date.day())
        }
        else {
            format!("{:02}-{:02} {:02}:{:02}",
                    date.month() as usize, date.day(),
                    date.hour(), date.minute())
        }
    }
}
