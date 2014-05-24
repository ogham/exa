use file::File;

pub enum SortField {
    Name, Size
}

pub struct Options {
    pub showInvisibles: bool,
    pub sortField: SortField,
}

impl SortField {
    pub fn from_word(word: StrBuf) -> SortField {
        match word.as_slice() {
            "name" => Name,
            "size" => Size,
            _ => fail!("Invalid sorting order"),
        }
    }

    fn sort(&self, files: &mut Vec<File>) {
        match *self {
            Name => files.sort_by(|a, b| a.name.cmp(&b.name)),
            Size => files.sort_by(|a, b| a.stat.size.cmp(&b.stat.size)),
        }
    }
}

impl Options {
    pub fn sort(&self, files: &mut Vec<File>) {
        self.sortField.sort(files);
    }

    pub fn show(&self, f: &File) -> bool {
        if self.showInvisibles {
            true
        } else {
            !f.name.starts_with(".")
        }
    }
}
