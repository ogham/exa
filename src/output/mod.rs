use output::file_name::FileStyle;
use style::Colours;

pub use self::cell::{TextCell, TextCellContents, DisplayWidth};
pub use self::escape::escape;

pub mod details;
pub mod file_name;
pub mod grid_details;
pub mod grid;
pub mod icons;
pub mod lines;
pub mod render;
pub mod table;
pub mod time;

mod cell;
mod escape;
mod tree;


/// The **view** contains all information about how to format output.
#[derive(Debug)]
pub struct View {
    pub mode: Mode,
    pub colours: Colours,
    pub style: FileStyle,
}


/// The **mode** is the “type” of output.
#[derive(Debug)]
pub enum Mode {
    Grid(grid::Options),
    Details(details::Options),
    GridDetails(grid_details::Options),
    Lines(lines::Options),
}
