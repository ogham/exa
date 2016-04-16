use ansi_term::ANSIStrings;

use fs::File;

use super::filename;
use super::colours::Colours;


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lines {
    pub colours: Colours,
}

/// The lines view literally just displays each file, line-by-line.
impl Lines {
    pub fn view(&self, files: Vec<File>) {
        for file in files {
            println!("{}", ANSIStrings(&filename(&file, &self.colours, true)));
        }
    }
}
