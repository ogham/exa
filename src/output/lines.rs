use std::io::{Write, Result as IOResult};

use ansi_term::{ANSIStrings, ANSIGenericString};

use crate::fs::File;
use crate::output::cell::TextCell;
use crate::output::file_name::{FileName, FileStyle};
use crate::output::icons::painted_icon;
use crate::style::Colours;


#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Options {
    pub icons: bool,
}

/// The lines view literally just displays each file, line-by-line.
pub struct Render<'a> {
    pub files: Vec<File<'a>>,
    pub colours: &'a Colours,
    pub style: &'a FileStyle,
    pub opts: &'a Options,
}

impl<'a> Render<'a> {
    pub fn render<W: Write>(&self, w: &mut W) -> IOResult<()> {
        for file in &self.files {
            let name_cell = self.render_file(file).paint();
            if self.opts.icons {
                // Create a TextCell for the icon then append the text to it
                let mut cell = TextCell::default();
                let icon = painted_icon(file, self.style);
                cell.push(ANSIGenericString::from(icon), 2);
                cell.append(name_cell.promote());
                writeln!(w, "{}", ANSIStrings(&cell))?;
            }
            else {
                writeln!(w, "{}", ANSIStrings(&name_cell))?;
            }
        }

        Ok(())
    }

    fn render_file<'f>(&self, file: &'f File<'a>) -> FileName<'f, 'a, Colours> {
        self.style.for_file(file, self.colours).with_link_paths()
    }
}
