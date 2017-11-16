use ansi_term::Style;
use datetime::{TimeZone, LocalDateTime};

use fs::fields as f;
use output::cell::TextCell;
use output::time::TimeFormat;


impl f::Time {
    pub fn render<C: Colours>(self, colours: &C,
                        tz: &Option<TimeZone>,
                         timestyle: &TimeFormat) -> TextCell {

        let age =  LocalDateTime::now().to_instant().seconds() - self.seconds;
        if let Some(ref tz) = *tz {
            let datestamp = timestyle.format_zoned(self, tz);
            return TextCell::paint(colours.stamp_age(age), datestamp)
        }
        else {
            let datestamp = timestyle.format_local(self);
            return TextCell::paint(colours.stamp_age(age), datestamp)
        }
    }

}

pub trait Colours {
    fn stamp_age (&self, age: i64) -> Style;
}

