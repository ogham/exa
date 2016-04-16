//! Wrapper types for the values returned from `File`s.
//!
//! The methods of `File` that return information about the entry on the
//! filesystem -- size, modification date, block count, or Git status -- used
//! to just return these as formatted strings, but this became inflexible once
//! customisable output styles landed.
//!
//! Instead, they will return a wrapper type from this module, which tags the
//! type with what field it is while containing the actual raw value.
//!
//! The `output::details` module, among others, uses these types to render and
//! display the information as formatted strings.

// C-style `blkcnt_t` types don’t follow Rust’s rules!
#![allow(non_camel_case_types)]


/// The type of a file’s block count.
pub type blkcnt_t = u64;

/// The type of a file’s group ID.
pub type gid_t = u32;

/// The type of a file’s inode.
pub type ino_t = u64;

/// The type of a file’s number of links.
pub type nlink_t = u64;

/// The type of a file’s timestamp (creation, modification, access, etc).
pub type time_t = i64;

/// The type of a file’s user ID.
pub type uid_t = u32;


/// The file’s base type, which gets displayed in the very first column of the
/// details output.
///
/// This type is set entirely by the filesystem, rather than relying on a
/// file’s contents. So “link” is a type, but “image” is just a type of
/// regular file. (See the `filetype` module for those checks.)
pub enum Type {
    File, Directory, Pipe, Link, Special,
}

impl Type {
    pub fn is_regular_file(&self) -> bool {
        match *self {
            Type::File  => true,
            _           => false,
        }
    }
}


/// The file’s Unix permission bitfield, with one entry per bit.
pub struct Permissions {
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


/// A file’s number of hard links on the filesystem.
///
/// Under Unix, a file can exist on the filesystem only once but appear in
/// multiple directories. However, it’s rare (but occasionally useful!) for a
/// regular file to have a link count greater than 1, so we highlight the
/// block count specifically for this case.
pub struct Links {

    /// The actual link count.
    pub count: nlink_t,

    /// Whether this file is a regular file with more than one hard link.
    pub multiple: bool,
}


/// A file’s inode. Every directory entry on a Unix filesystem has an inode,
/// including directories and links, so this is applicable to everything exa
/// can deal with.
pub struct Inode(pub ino_t);


/// The number of blocks that a file takes up on the filesystem, if any.
pub enum Blocks {

    /// This file has the given number of blocks.
    Some(blkcnt_t),

    /// This file isn’t of a type that can take up blocks.
    None,
}


/// The ID of the user that owns a file. This will only ever be a number;
/// looking up the username is done in the `display` module.
pub struct User(pub uid_t);

/// The ID of the group that a file belongs to.
pub struct Group(pub gid_t);


/// A file’s size, in bytes. This is usually formatted by the `number_prefix`
/// crate into something human-readable.
pub enum Size {

    /// This file has a defined size.
    Some(u64),

    /// This file has no size, or has a size but we aren’t interested in it.
    ///
    /// Under Unix, directory entries that aren’t regular files will still
    /// have a file size. For example, a directory will just contain a list of
    /// its files as its “contents” and will be specially flagged as being a
    /// directory, rather than a file. However, seeing the “file size” of this
    /// data is rarely useful -- I can’t think of a time when I’ve seen it and
    /// learnt something. So we discard it and just output “-” instead.
    ///
    /// See this answer for more: http://unix.stackexchange.com/a/68266
    None,
}


/// One of a file’s timestamps (created, accessed, or modified).
pub struct Time(pub time_t);


/// A file’s status in a Git repository. Whether a file is in a repository or
/// not is handled by the Git module, rather than having a “null” variant in
/// this enum.
pub enum GitStatus {

    /// This file hasn’t changed since the last commit.
    NotModified,

    /// This file didn’t exist for the last commit, and is not specified in
    /// the ignored files list.
    New,

    /// A file that’s been modified since the last commit.
    Modified,

    /// A deleted file. This can’t ever be shown, but it’s here anyway!
    Deleted,

    /// A file that Git has tracked a rename for.
    Renamed,

    /// A file that’s had its type (such as the file permissions) changed.
    TypeChange,
}

/// A file’s complete Git status. It’s possible to make changes to a file, add
/// it to the staging area, then make *more* changes, so we need to list each
/// file’s status for both of these.
pub struct Git {
    pub staged:   GitStatus,
    pub unstaged: GitStatus,
}

impl Git {

    /// Create a Git status for a file with nothing done to it.
    pub fn empty() -> Git {
        Git { staged: GitStatus::NotModified, unstaged: GitStatus::NotModified }
    }
}
