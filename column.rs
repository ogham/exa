pub enum Column {
    Permissions,
    FileName,
    FileSize(bool),
    User(u64),
    Group,
    HardLinks,
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
            _           => Left,
        }
    }
}

// An Alignment is used to pad a string to a certain length, letting
// it pick which end it puts the text on. The length of the string is
// passed in specifically because it needs to be the *unformatted*
// length, rather than just the number of characters.

impl Alignment {
    pub fn pad_string(&self, string: &String, string_length: uint, width: uint) -> String {
        let mut str = String::new();
        match *self {
            Left => {
                str.push_str(string.as_slice());
                for _ in range(string_length, width) {
                    str.push_char(' ');
                }
            }

            Right => {
                for _ in range(string_length, width) {
                    str.push_char(' ');
                }
                str.push_str(string.as_slice());
            },
        }
        return str;
    }
}

