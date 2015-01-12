use std::iter::repeat;

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

// Each column can pick its own alignment. Usually, numbers are
// right-aligned, and text is left-aligned.

pub enum Alignment {
    Left, Right,
}

impl Copy for Alignment { }

impl Column {
    pub fn alignment(&self) -> Alignment {
        match *self {
            Column::FileSize(_) => Alignment::Right,
            Column::HardLinks   => Alignment::Right,
            Column::Inode       => Alignment::Right,
            Column::Blocks      => Alignment::Right,
            _                   => Alignment::Left,
        }
    }

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

fn spaces(length: usize) -> String {
    repeat(" ").take(length).collect()
}

// An Alignment is used to pad a string to a certain length, letting
// it pick which end it puts the text on. It takes the amount of
// padding to apply, rather than the width the text should end up,
// because these strings are usually full of control characters.

impl Alignment {
    pub fn pad_string(&self, string: &String, padding: usize) -> String {
        match *self {
            Alignment::Left  => format!("{}{}", string, spaces(padding).as_slice()),
            Alignment::Right => format!("{}{}", spaces(padding), string.as_slice()),
        }
    }
}
