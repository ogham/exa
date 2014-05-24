use file::File;
use std::cmp::lexical_ordering;

pub enum SortField {
    Name, Extension, Size
}

pub struct Options {
    pub showInvisibles: bool,
    pub sortField: SortField,
    pub reverse: bool,
}

impl SortField {
    pub fn from_word(word: StrBuf) -> SortField {
        match word.as_slice() {
            "name" => Name,
            "size" => Size,
            "ext" => Extension,
            _ => fail!("Invalid sorting order"),
        }
    }

    fn sort(&self, files: &mut Vec<File>) {
        match *self {
            Name => files.sort_by(|a, b| a.name.cmp(&b.name)),
            Size => files.sort_by(|a, b| a.stat.size.cmp(&b.stat.size)),
            Extension => files.sort_by(|a, b| {
                let exts = a.ext.cmp(&b.ext);
                let names = a.name.cmp(&b.name);
                lexical_ordering(exts, names)
            }),
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
