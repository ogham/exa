use std::io::{self, Write};

use ansi_term::ANSIStrings;

use crate::fs::File;
use crate::fs::filter::FileFilter;
use crate::output::cell::TextCellContents;
use crate::output::file_name::{Options as FileStyle};
use crate::theme::Theme;


/// The lines view literally just displays each file, line-by-line.
pub struct Render<'a> {
    pub files: Vec<File<'a>>,
    pub theme: &'a Theme,
    pub file_style: &'a FileStyle,
    pub filter: &'a FileFilter,
}

impl<'a> Render<'a> {
    pub fn render<W: Write>(mut self, w: &mut W) -> io::Result<()> {
        self.filter.sort_files(&mut self.files);
        for file in &self.files {
            let name_cell = self.render_file(file);
            writeln!(w, "{}", ANSIStrings(&name_cell))?;
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
