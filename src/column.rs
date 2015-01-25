use std::iter::repeat;

use ansi_term::Style;

#[derive(PartialEq, Show)]
pub enum Column {
    Permissions,
    FileName,
    FileSize(SizeFormat),
    Blocks,
    User,
    Group,
    HardLinks,
    Inode,
}

impl Copy for Column { }

#[derive(PartialEq, Show)]
pub enum SizeFormat {
    DecimalBytes,
    BinaryBytes,
    JustBytes,
}

impl Copy for SizeFormat { }

/// Each column can pick its own **Alignment**. Usually, numbers are
/// right-aligned, and text is left-aligned.
pub enum Alignment {
    Left, Right,
}

impl Copy for Alignment { }

impl Column {

    /// Get the alignment this column should use.
    pub fn alignment(&self) -> Alignment {
        match *self {
            Column::FileSize(_) => Alignment::Right,
            Column::HardLinks   => Alignment::Right,
            Column::Inode       => Alignment::Right,
            Column::Blocks      => Alignment::Right,
            _                   => Alignment::Left,
        }
    }

    /// Get the text that should be printed at the top, when the user elects
    /// to have a header row printed.
    pub fn header(&self) -> &'static str {
        match *self {
            Column::Permissions => "Permissions",
            Column::FileName    => "Name",
            Column::FileSize(_) => "Size",
            Column::Blocks      => "Blocks",
            Column::User        => "User",
            Column::Group       => "Group",
            Column::HardLinks   => "Links",
            Column::Inode       => "inode",
        }
    }
}

/// Pad a string with the given number of spaces.
fn spaces(length: usize) -> String {
    repeat(" ").take(length).collect()
}

impl Alignment {
    /// Pad a string with the given alignment and number of spaces.
    ///
    /// This doesn't take the width the string *should* be, rather the number
    /// of spaces to add: this is because the strings are usually full of
    /// invisible control characters, so getting the displayed width of the
    /// string is not as simple as just getting its length.
    pub fn pad_string(&self, string: &String, padding: usize) -> String {
        match *self {
            Alignment::Left  => format!("{}{}", string, spaces(padding).as_slice()),
            Alignment::Right => format!("{}{}", spaces(padding), string.as_slice()),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Cell {
    pub length: usize,
    pub text: String,
}

impl Cell {
    pub fn paint(style: Style, string: &str) -> Cell {
        Cell {
            text: style.paint(string).to_string(),
            length: string.len(),
        }
    }
}
