use output::cell::TextCell;
use output::colours::Colours;
use fs::fields as f;

use datetime::{LocalDateTime, TimeZone, DatePiece};
use datetime::fmt::DateFormat;
use locale;


#[allow(trivial_numeric_casts)]
impl f::Time {
    pub fn render(&self, colours: &Colours, tz: &Option<TimeZone>,
                          date_and_time: &DateFormat<'static>, date_and_year: &DateFormat<'static>,
                          time: &locale::Time, current_year: i64) -> TextCell {

        // TODO(ogham): This method needs some serious de-duping!
        // zoned and local times have different types at the moment,
        // so it's tricky.

        if let Some(ref tz) = *tz {
            let date = tz.to_zoned(LocalDateTime::at(self.0 as i64));

            let datestamp = if date.year() == current_year {
                date_and_time.format(&date, time)
            }
            else {
                date_and_year.format(&date, time)
            };

            TextCell::paint(colours.date, datestamp)
        }
        else {
            let date = LocalDateTime::at(self.0 as i64);

            let datestamp = if date.year() == current_year {
                date_and_time.format(&date, time)
            }
            else {
                date_and_year.format(&date, time)
            };

            TextCell::paint(colours.date, datestamp)
        }
    }
}
