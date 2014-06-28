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
        let mut str = String::new();
        match *self {
            Left => {
                str.push_str(string.as_slice());
                for _ in range(0, padding) {
                    str.push_char(' ');
                }
            }

            Right => {
                for _ in range(0, padding) {
                    str.push_char(' ');
                }
                str.push_str(string.as_slice());
            },
        }
        return str;
    }
}

