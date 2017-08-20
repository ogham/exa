use output::file_name::FileStyle;

pub use self::cell::{TextCell, TextCellContents, DisplayWidth};
pub use self::colours::Colours;
pub use self::escape::escape;
pub use self::lsc::LSColors;

pub mod details;
pub mod file_name;
pub mod grid_details;
pub mod grid;
pub mod lines;
pub mod table;
pub mod time;

mod cell;
mod colours;
mod escape;
mod lsc;
mod render;
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
    Lines,
}
