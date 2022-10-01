pub use self::cell::{TextCell, TextCellContents, DisplayWidth};
pub use self::escape::escape;

pub mod details;
pub mod file_name;
pub mod grid;
pub mod grid_details;
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
    pub width: TerminalWidth,
    pub file_style: file_name::Options,
}


/// The **mode** is the “type” of output.
#[derive(PartialEq, Eq, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Mode {
    Grid(grid::Options),
    Details(details::Options),
    GridDetails(grid_details::Options),
    Lines,
}


/// The width of the terminal requested by the user.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum TerminalWidth {

    /// The user requested this specific number of columns.
    Set(usize),

    /// Look up the terminal size at runtime.
    Automatic,
}

impl TerminalWidth {
    pub fn actual_terminal_width(self) -> Option<usize> {
        // All of stdin, stdout, and stderr could not be connected to a
        // terminal, but we’re only interested in stdout because it’s
        // where the output goes.

        match self {
            Self::Set(width)  => Some(width),
            Self::Automatic   => terminal_size::terminal_size().map(|(w, _)| w.0.into()),
        }
    }
}
