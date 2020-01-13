use datetime::TimeZone;
use ansi_term::Style;

use output::cell::TextCell;
use output::time::TimeFormat;


pub trait Render {
    fn render(self, style: Style,
                        tz: &Option<TimeZone>,
                        format: &TimeFormat) -> TextCell;
}

impl Render for Option<std::time::Duration> {
    fn render(self, style: Style,
                        tz: &Option<TimeZone>,
                        format: &TimeFormat) -> TextCell {
        if let Some(duration) = self {
            if let Some(ref tz) = *tz {
                let datestamp = format.format_zoned(duration, tz);
                TextCell::paint(style, datestamp)
            }
            else {
                let datestamp = format.format_local(duration);
                TextCell::paint(style, datestamp)
            }
        }
        else {
            TextCell::paint(style, String::from("-"))
        }
    }
}
