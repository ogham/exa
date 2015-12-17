use colours::Colours;
use file::File;
use filetype::file_colour;

use term_grid as grid;


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
            grid.add(grid::Cell {
                contents:  file_colour(&self.colours, file).paint(&*file.name).to_string(),
                width:     *file.file_name_width(),
            });
        }

        if let Some(display) = grid.fit_into_width(self.console_width) {
            print!("{}", display);
        }
        else {
            // File names too long for a grid - drop down to just listing them!
            for file in files.iter() {
                println!("{}", file_colour(&self.colours, file).paint(&*file.name));
            }
        }
    }
}
