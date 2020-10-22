use std::io::{self, Write};

use term_grid as tg;

use crate::fs::File;
use crate::output::cell::DisplayWidth;
use crate::output::file_name::Options as FileStyle;
use crate::output::icons::painted_icon;
use crate::output::lines;
use crate::theme::Theme;


#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Options {
    pub across: bool,
    pub icons: bool,
}

impl Options {
    pub fn direction(self) -> tg::Direction {
        if self.across { tg::Direction::LeftToRight }
                  else { tg::Direction::TopToBottom }
    }

    pub fn to_lines_options(self) -> lines::Options {
        lines::Options {
            icons: self.icons
        }
    }
}


pub struct Render<'a> {
    pub files: Vec<File<'a>>,
    pub theme: &'a Theme,
    pub file_style: &'a FileStyle,
    pub opts: &'a Options,
    pub console_width: usize,
}

impl<'a> Render<'a> {
    pub fn render<W: Write>(&self, w: &mut W) -> io::Result<()> {
        let mut grid = tg::Grid::new(tg::GridOptions {
            direction:  self.opts.direction(),
            filling:    tg::Filling::Spaces(2),
        });

        grid.reserve(self.files.len());

        for file in &self.files {
            let icon = if self.opts.icons { Some(painted_icon(file, self.theme)) }
                                     else { None };

            let filename = self.file_style.for_file(file, self.theme).paint();

            let width = if self.opts.icons { DisplayWidth::from(2) + filename.width() }
                                      else { filename.width() };

            grid.add(tg::Cell {
                contents:  format!("{}{}", &icon.unwrap_or_default(), filename.strings()),
                width:     *width,
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
                if self.opts.icons {
                    write!(w, "{}", painted_icon(file, self.theme))?;
                }

                let name_cell = self.file_style.for_file(file, self.theme).paint();
                writeln!(w, "{}", name_cell.strings())?;
            }

            Ok(())
        }
    }
}
