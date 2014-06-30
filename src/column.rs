pub enum Column {
    Permissions,
    FileName,
    FileSize(bool),
    Blocks,
    User,
    Group,
    HardLinks,
    Inode,
}

// Each column can pick its own alignment. Usually, numbers are
// right-aligned, and text is left-aligned.

pub enum Alignment {
    Left, Right,
}

impl Column {
    pub fn alignment(&self) -> Alignment {
        match *self {
            FileSize(_) => Right,
            HardLinks   => Right,
            Inode       => Right,
            Blocks      => Right,
            _           => Left,
        }
    }

    pub fn header(&self) -> &'static str {
        match *self {
            Permissions => "Permissions",
            FileName => "Name",
            FileSize(_) => "Size",
            Blocks => "Blocks",
            User => "User",
            Group => "Group",
            HardLinks => "Links",
            Inode => "inode",
        }
    }
}

// An Alignment is used to pad a string to a certain length, letting
// it pick which end it puts the text on. It takes the amount of
// padding to apply, rather than the width the text should end up,
// because these strings are usually full of control characters.

impl Alignment {
    pub fn pad_string(&self, string: &String, padding: uint) -> String {
        match *self {
            Left => string.clone().append(" ".to_string().repeat(padding).as_slice()),
            Right => " ".to_string().repeat(padding).append(string.as_slice()),
        }
    }
}

