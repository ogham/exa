use term_grid as grid;

use fs::File;
use output::DisplayWidth;
use output::colours::Colours;
use super::filename;


#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Grid {
    pub across: bool,
    pub console_width: usize,
    pub colours: Colours,
}

impl Grid {
    pub fn view(&self, files: &[File]) {
        let direction = if self.across { grid::Direction::LeftToRight }
                                  else { grid::Direction::TopToBottom };

        let mut grid = grid::Grid::new(grid::GridOptions {
            direction:  direction,
            filling:    grid::Filling::Spaces(2),
        });

        grid.reserve(files.len());

        for file in files.iter() {
            let mut width = DisplayWidth::from(&*file.name);

            if file.dir.is_none() {
                if let Some(ref parent) = file.path.parent() {
                    width = width + 1 + DisplayWidth::from(parent.to_string_lossy().as_ref());
                }
            }

            grid.add(grid::Cell {
                contents:  filename(file, &self.colours, false).strings().to_string(),
                width:     *width,
            });
        }

        if let Some(display) = grid.fit_into_width(self.console_width) {
            print!("{}", display);
        }
        else {
            // File names too long for a grid - drop down to just listing them!
            for file in files.iter() {
                println!("{}", filename(file, &self.colours, false).strings());
            }
        }
    }
}
