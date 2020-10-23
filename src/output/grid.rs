use std::io::{self, Write};

use term_grid as tg;

use crate::fs::File;
use crate::fs::filter::FileFilter;
use crate::output::file_name::Options as FileStyle;
use crate::theme::Theme;


#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Options {
    pub across: bool,
}

impl Options {
    pub fn direction(self) -> tg::Direction {
        if self.across { tg::Direction::LeftToRight }
                  else { tg::Direction::TopToBottom }
    }
}


pub struct Render<'a> {
    pub files: Vec<File<'a>>,
    pub theme: &'a Theme,
    pub file_style: &'a FileStyle,
    pub opts: &'a Options,
    pub console_width: usize,
    pub filter: &'a FileFilter,
}

impl<'a> Render<'a> {
    pub fn render<W: Write>(mut self, w: &mut W) -> io::Result<()> {
        let mut grid = tg::Grid::new(tg::GridOptions {
            direction:  self.opts.direction(),
            filling:    tg::Filling::Spaces(2),
        });

        grid.reserve(self.files.len());

        self.filter.sort_files(&mut self.files);
        for file in &self.files {
            let filename = self.file_style.for_file(file, self.theme).paint();

            grid.add(tg::Cell {
                contents:  filename.strings().to_string(),
                width:     *filename.width(),
            });
        }

        if let Some(display) = grid.fit_into_width(self.console_width) {
            write!(w, "{}", display)
        }
        else {
            // File names too long for a grid - drop down to just listing them!
            // This isn’t *quite* the same as the lines view, which also
            // displays full link paths.
            for file in &self.files {
                let name_cell = self.file_style.for_file(file, self.theme).paint();
                writeln!(w, "{}", name_cell.strings())?;
            }

            Ok(())
        }
    }
}
