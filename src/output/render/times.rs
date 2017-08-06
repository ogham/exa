use datetime::{TimeZone, LocalDateTime};

use fs::fields as f;
use output::cell::TextCell;
use output::colours::Colours;

use fs::fields as f;
use output::cell::TextCell;
use output::colours::Colours;
use output::time::TimeFormat;


impl f::Time {
    pub fn render(self, colours: &Colours,
                        tz: &Option<TimeZone>,
                         timestyle: &TimeFormat) -> TextCell {

        let age =  LocalDateTime::now().to_instant().seconds() - self.seconds;
        if let Some(ref tz) = *tz {
            let datestamp = timestyle.format_zoned(self, tz);

        let age =  LocalDateTime::now().to_instant().seconds() - self.seconds;
        if let Some(ref tz) = *tz {
        if let Some(ref tz) = *tz {
            let datestamp = style.format_zoned(self, tz);
        }
        else {
            let datestamp = timestyle.format_local(self);
            TextCell::paint(colours.stamp_age(age), datestamp)
        }
    }

}

        }
    }
}
