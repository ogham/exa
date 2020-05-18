use datetime::TimeZone;
use ansi_term::Style;

use crate::output::cell::TextCell;
use crate::output::time::TimeFormat;


pub trait Render {
    fn render(self, style: Style,
                        tz: &Option<TimeZone>,
                        format: &TimeFormat) -> TextCell;
}

impl Render for std::time::SystemTime {
    fn render(self, style: Style,
                        tz: &Option<TimeZone>,
                        format: &TimeFormat) -> TextCell {

        if let Some(ref tz) = *tz {
            let datestamp = format.format_zoned(self, tz);
            TextCell::paint(style, datestamp)
        }
        else {
            let datestamp = format.format_local(self);
            TextCell::paint(style, datestamp)
        }
    }
}
