use std::io::{Write, Result as IOResult};

use ansi_term::ANSIStrings;

use fs::File;

use output::file_name::{FileName, LinkStyle, Classify};
use super::colours::Colours;


/// The lines view literally just displays each file, line-by-line.
pub struct Render<'a> {
    pub files: Vec<File<'a>>,
    pub colours: &'a Colours,
    pub classify: Classify,
}

impl<'a> Render<'a> {
    pub fn render<W: Write>(&self, w: &mut W) -> IOResult<()> {
        for file in &self.files {
            let name_cell = self.render_file(file).paint();
            writeln!(w, "{}", ANSIStrings(&name_cell))?;
        }

        Ok(())
    }

    fn render_file<'f>(&self, file: &'f File<'a>) -> FileName<'f, 'a> {
        FileName::new(file, LinkStyle::FullLinkPaths, self.classify, self.colours)
    }
}
