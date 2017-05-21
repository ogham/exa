mod groups;
mod permissions;
mod size;
mod users;

use output::cell::{TextCell, DisplayWidth};
use output::colours::Colours;
use fs::fields as f;

use datetime::{LocalDateTime, TimeZone, DatePiece};
use datetime::fmt::DateFormat;
use locale;


impl f::Links {
    pub fn render(&self, colours: &Colours, numeric: &locale::Numeric) -> TextCell {
        let style = if self.multiple { colours.links.multi_link_file }
                                else { colours.links.normal };

        TextCell::paint(style, numeric.format_int(self.count))
    }
}


impl f::Blocks {
    pub fn render(&self, colours: &Colours) -> TextCell {
        match *self {
            f::Blocks::Some(ref blk)  => TextCell::paint(colours.blocks, blk.to_string()),
            f::Blocks::None           => TextCell::blank(colours.punctuation),
        }
    }
}


impl f::Inode {
    pub fn render(&self, colours: &Colours) -> TextCell {
        TextCell::paint(colours.inode, self.0.to_string())
    }
}


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


impl f::Git {
    pub fn render(&self, colours: &Colours) -> TextCell {
        let git_char = |status| match status {
            &f::GitStatus::NotModified  => colours.punctuation.paint("-"),
            &f::GitStatus::New          => colours.git.new.paint("N"),
            &f::GitStatus::Modified     => colours.git.modified.paint("M"),
            &f::GitStatus::Deleted      => colours.git.deleted.paint("D"),
            &f::GitStatus::Renamed      => colours.git.renamed.paint("R"),
            &f::GitStatus::TypeChange   => colours.git.typechange.paint("T"),
        };

        TextCell {
            width: DisplayWidth::from(2),
            contents: vec![
                git_char(&self.staged),
                git_char(&self.unstaged)
            ].into(),
        }
    }
}
