use std::io::{Write, Result as IOResult};

use term_grid as grid;

use fs::File;
use output::colours::Colours;
use output::file_name::{FileName, LinkStyle, Classify};


#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Grid {
    pub across: bool,
    pub console_width: usize,
}

impl Grid {
    pub fn view<W: Write>(&self, files: &[File], w: &mut W, colours: &Colours, classify: Classify) -> IOResult<()> {
        let direction = if self.across { grid::Direction::LeftToRight }
                                  else { grid::Direction::TopToBottom };

        let mut grid = grid::Grid::new(grid::GridOptions {
            direction:  direction,
            filling:    grid::Filling::Spaces(2),
        });

        grid.reserve(files.len());

        for file in files.iter() {
            let filename = FileName::new(file, LinkStyle::JustFilenames, classify, colours).paint();
            let width = filename.width();

            grid.add(grid::Cell {
                contents:  filename.strings().to_string(),
                width:     *width,
            });
        }

        if let Some(display) = grid.fit_into_width(self.console_width) {
            write!(w, "{}", display)
        }
        else {
            // File names too long for a grid - drop down to just listing them!
            for file in files.iter() {
                let name_cell = FileName::new(file, LinkStyle::JustFilenames, classify, colours).paint();
                writeln!(w, "{}", name_cell.strings())?;
            }
            Ok(())
        }
    }
}
