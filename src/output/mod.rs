use ansi_term::Style;

use colours::Colours;
use file::File;
use filetype::file_colour;

pub use self::cell::{TextCell, TextCellContents, DisplayWidth};
pub use self::details::Details;
pub use self::grid::Grid;
pub use self::lines::Lines;
pub use self::grid_details::GridDetails;

mod grid;
pub mod details;
mod lines;
mod grid_details;
pub mod column;
mod cell;


pub fn filename(file: File, colours: &Colours, links: bool) -> TextCellContents {
    if links && file.is_link() {
        symlink_filename(file, colours)
    }
    else {
        vec![
            file_colour(colours, &file).paint(file.name)
        ]
    }
}

fn symlink_filename(file: File, colours: &Colours) -> TextCellContents {
    match file.link_target() {
        Ok(target) => vec![
            file_colour(colours, &file).paint(file.name),
            Style::default().paint(" "),
            colours.punctuation.paint("->"),
            Style::default().paint(" "),
            colours.symlink_path.paint(target.path_prefix()),
            file_colour(colours, &target).paint(target.name)
        ],

        Err(filename) => vec![
            file_colour(colours, &file).paint(file.name),
            Style::default().paint(" "),
            colours.broken_arrow.paint("->"),
            Style::default().paint(" "),
            colours.broken_filename.paint(filename),
        ],
    }
}
