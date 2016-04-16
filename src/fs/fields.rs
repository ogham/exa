#![allow(non_camel_case_types)]

/// Wrapper types for the values returned from `File` objects.
///
/// The methods of `File` don't return formatted strings; neither do they
/// return raw numbers representing timestamps or user IDs. Instead, they will
/// return an object in this `fields` module. These objects are later rendered
/// into formatted strings in the `output/details` module.
pub type blkcnt_t = u64;
pub type gid_t = u32;
pub type ino_t = u64;
pub type nlink_t = u64;
pub type time_t = i64;
pub type uid_t = u32;

pub enum Type {
    File, Directory, Pipe, Link, Special,
}

pub struct Permissions {
    pub file_type:      Type,
    pub user_read:      bool,
    pub user_write:     bool,
    pub user_execute:   bool,
    pub group_read:     bool,
    pub group_write:    bool,
    pub group_execute:  bool,
    pub other_read:     bool,
    pub other_write:    bool,
    pub other_execute:  bool,
}

pub struct Links {
    pub count: nlink_t,
    pub multiple: bool,
}

pub struct Inode(pub ino_t);

pub enum Blocks {
    Some(blkcnt_t),
    None,
}

pub struct User(pub uid_t);

pub struct Group(pub gid_t);

pub enum Size {
    Some(u64),
    None,
}

pub struct Time(pub time_t);

pub enum GitStatus {
    NotModified,
    New,
    Modified,
    Deleted,
    Renamed,
    TypeChange,
}

pub struct Git {
    pub staged:   GitStatus,
    pub unstaged: GitStatus,
}

impl Git {
    pub fn empty() -> Git {
        Git { staged: GitStatus::NotModified, unstaged: GitStatus::NotModified }
    }
}
