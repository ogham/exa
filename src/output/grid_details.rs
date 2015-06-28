use std::convert;
use std::iter::repeat;

use term_grid as grid;

use column::Cell;
use dir::Dir;
use file::File;
use output::details::{Details, Table};
use output::grid::Grid;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct GridDetails {
    pub grid: Grid,
    pub details: Details,
}

impl GridDetails {
    pub fn view(&self, dir: Option<&Dir>, files: &[File]) {

        let columns = 2;

        let make_table = || {
            let mut table = Table::with_options(self.details.colours, self.details.columns.for_dir(dir));
            if self.details.header { table.add_header() }
            table
        };

        let mut tables: Vec<_> = repeat(()).map(|_| make_table()).take(columns).collect();

        for (i, file) in files.iter().enumerate() {
            tables[i % columns].add_file(file, 0, false, false);
        }

        let direction = if self.grid.across { grid::Direction::LeftToRight }
                                       else { grid::Direction::TopToBottom };

        let mut grid = grid::Grid::new(grid::GridOptions {
            direction:        direction,
            separator_width:  2,
        });

        for table in tables {
            for cell in table.print_table(false, false).into_iter() {
                grid.add(cell.into());
            }
        }

        print!("{}", grid.fit_into_columns(columns));
    }
}

impl convert::From<Cell> for grid::Cell {
    fn from(input: Cell) -> Self {
        grid::Cell {
            contents: input.text,
            width:    input.length,
        }
    }
}
