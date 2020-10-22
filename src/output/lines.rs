use std::io::{self, Write};

use ansi_term::{ANSIStrings, ANSIGenericString};

use crate::fs::File;
use crate::output::cell::{TextCell, TextCellContents};
use crate::output::file_name::{Options as FileStyle};
use crate::output::icons::painted_icon;
use crate::theme::Theme;


#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Options {
    pub icons: bool,
}

/// The lines view literally just displays each file, line-by-line.
pub struct Render<'a> {
    pub files: Vec<File<'a>>,
    pub theme: &'a Theme,
    pub file_style: &'a FileStyle,
    pub opts: &'a Options,
}

impl<'a> Render<'a> {
    pub fn render<W: Write>(&self, w: &mut W) -> io::Result<()> {
        for file in &self.files {
            let name_cell = self.render_file(file);
            if self.opts.icons {
                // Create a TextCell for the icon then append the text to it
                let mut cell = TextCell::default();
                let icon = painted_icon(file, self.theme);
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

    fn render_file<'f>(&self, file: &'f File<'a>) -> TextCellContents {
        self.file_style
            .for_file(file, self.theme)
            .with_link_paths()
            .paint()
    }
}
