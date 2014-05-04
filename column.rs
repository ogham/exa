pub enum Column {
    Permissions,
    FileName,
    FileSize(bool),
}

pub fn defaultColumns() -> ~[Column] {
    return ~[
        Permissions,
        FileSize(false),
        FileName,
    ];
}
