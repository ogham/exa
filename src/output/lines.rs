use std::io::{Write, Result as IOResult};

use ansi_term::ANSIStrings;

use fs::File;

use output::file_name::{FileName, LinkStyle};
use super::colours::Colours;


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lines {
    pub colours: Colours,
    pub classify: bool,
}

/// The lines view literally just displays each file, line-by-line.
impl Lines {
    pub fn view<W: Write>(&self, files: Vec<File>, w: &mut W) -> IOResult<()> {
        for file in files {
            let name_cell = FileName::new(&file, LinkStyle::FullLinkPaths, &self.colours).paint(self.classify);
            writeln!(w, "{}", ANSIStrings(&name_cell))?;
        }
        Ok(())
    }
}
