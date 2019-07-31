use std::io::{Write, Result as IOResult};

use term_grid as tg;

use fs::File;
use style::Colours;
use output::file_name::FileStyle;
use output::icons::painted_icon;
use output::cell::DisplayWidth;


#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Options {
    pub across: bool,
    pub console_width: usize,
    pub icons: bool,
}

impl Options {
    pub fn direction(&self) -> tg::Direction {
        if self.across { tg::Direction::LeftToRight }
                  else { tg::Direction::TopToBottom }
    }
}


pub struct Render<'a> {
    pub files: Vec<File<'a>>,
    pub colours: &'a Colours,
    pub style: &'a FileStyle,
    pub opts: &'a Options,
}

impl<'a> Render<'a> {
    pub fn render<W: Write>(&self, w: &mut W) -> IOResult<()> {
        let mut grid = tg::Grid::new(tg::GridOptions {
            direction:  self.opts.direction(),
            filling:    tg::Filling::Spaces(2),
        });

        grid.reserve(self.files.len());

        for file in &self.files {
            let icon = if self.opts.icons { Some(painted_icon(&file, &self.style)) } else { None };
            let filename = self.style.for_file(file, self.colours).paint();
            let width = if self.opts.icons {
                DisplayWidth::from(2) + filename.width()
            } else {
                filename.width()
            };

            grid.add(tg::Cell {
                contents:  format!("{icon}{filename}", icon=&icon.unwrap_or("".to_string()), filename=filename.strings().to_string()),
                width:     *width,
            });
        }

        if let Some(display) = grid.fit_into_width(self.opts.console_width) {
            write!(w, "{}", display)
        }
        else {
            // File names too long for a grid - drop down to just listing them!
            // This isn’t *quite* the same as the lines view, which also
            // displays full link paths.
            for file in &self.files {
                if self.opts.icons {
                    write!(w, "{}", painted_icon(&file, &self.style))?;
                }
                let name_cell = self.style.for_file(file, self.colours).paint();
                writeln!(w, "{}", name_cell.strings())?;
            }
            Ok(())
        }
    }
}
