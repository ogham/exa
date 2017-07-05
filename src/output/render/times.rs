use datetime::TimeZone;

use fs::fields as f;
use output::cell::TextCell;
use output::colours::Colours;
use output::time::TimeFormat;


impl f::Time {
    pub fn render(self, colours: &Colours,
                         tz: &Option<TimeZone>,
                         style: &TimeFormat) -> TextCell {

        if let Some(ref tz) = *tz {
            let datestamp = style.format_zoned(self, tz);
            TextCell::paint(colours.date, datestamp)
        }
        else {
            let datestamp = style.format_local(self);
            TextCell::paint(colours.date, datestamp)
        }
    }
}

