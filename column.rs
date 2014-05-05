pub enum Column {
    Permissions,
    FileName,
    FileSize(bool),
    User,
    Group,
}

pub fn defaultColumns() -> ~[Column] {
    return ~[
        Permissions,
        FileSize(false),
        User,
        Group,
        FileName,
    ];
}
