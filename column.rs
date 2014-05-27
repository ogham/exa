pub enum Column {
    Permissions,
    FileName,
    FileSize(bool),
    User(u64),
    Group,
}
