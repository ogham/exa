use ansi_term::Style;
use unicode_width::UnicodeWidthStr;

use options::{SizeFormat, TimeType};


#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Column {
    Permissions,
    FileSize(SizeFormat),
    Timestamp(TimeType),
    Blocks,
    User,
    Group,
    HardLinks,
    Inode,

    GitStatus,
}

/// Each column can pick its own **Alignment**. Usually, numbers are
/// right-aligned, and text is left-aligned.
#[derive(Copy, Clone)]
pub enum Alignment {
    Left, Right,
}

impl Column {

    /// Get the alignment this column should use.
    pub fn alignment(&self) -> Alignment {
        match *self {
            Column::FileSize(_) => Alignment::Right,
            Column::HardLinks   => Alignment::Right,
            Column::Inode       => Alignment::Right,
            Column::Blocks      => Alignment::Right,
            Column::GitStatus   => Alignment::Right,
            _                   => Alignment::Left,
        }
    }

    /// Get the text that should be printed at the top, when the user elects
    /// to have a header row printed.
    pub fn header(&self) -> &'static str {
        match *self {
            Column::Permissions   => "Permissions",
            Column::FileSize(_)   => "Size",
            Column::Timestamp(t)  => t.header(),
            Column::Blocks        => "Blocks",
            Column::User          => "User",
            Column::Group         => "Group",
            Column::HardLinks     => "Links",
            Column::Inode         => "inode",
            Column::GitStatus     => "Git",
        }
    }
}


#[derive(PartialEq, Debug, Clone)]
pub struct Cell {
    pub length: usize,
    pub text: String,
}

impl Cell {
    pub fn empty() -> Cell {
        Cell {
            text: String::new(),
            length: 0,
        }
    }

    pub fn paint(style: Style, string: &str) -> Cell {
        Cell {
            text: style.paint(string).to_string(),
            length: UnicodeWidthStr::width(string),
        }
    }

    pub fn add_spaces(&mut self, count: usize) {
        self.length += count;
        for _ in 0 .. count {
            self.text.push(' ');
        }
    }

    pub fn append(&mut self, other: &Cell) {
        self.length += other.length;
        self.text.push_str(&*other.text);
    }
}
