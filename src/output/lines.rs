use std::io::{Write, Result as IOResult};

use ansi_term::ANSIStrings;

use fs::File;

use output::file_name::{FileName, LinkStyle, Classify};
use super::colours::Colours;


/// The lines view literally just displays each file, line-by-line.
pub fn view<W: Write>(files: Vec<File>, w: &mut W, colours: &Colours, classify: Classify) -> IOResult<()> {
    for file in files {
        let name_cell = FileName::new(&file, LinkStyle::FullLinkPaths, classify, colours).paint();
        writeln!(w, "{}", ANSIStrings(&name_cell))?;
    }
    Ok(())
}
