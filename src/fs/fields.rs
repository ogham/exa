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
#![allow(clippy::struct_excessive_bools)]


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
///
/// Its ordering is used when sorting by type.
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum Type {
    Directory,
    File,
    Link,
    Pipe,
    Socket,
    CharDevice,
    BlockDevice,
    Special,
}

impl Type {
    pub fn is_regular_file(self) -> bool {
        matches!(self, Self::File)
    }
}


/// The file’s Unix permission bitfield, with one entry per bit.
#[derive(Copy, Clone)]
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

    pub sticky:         bool,
    pub setgid:         bool,
    pub setuid:         bool,
}

/// The three pieces of information that are displayed as a single column in
/// the details view. These values are fused together to make the output a
/// little more compressed.
#[derive(Copy, Clone)]
pub struct PermissionsPlus {
    pub file_type:   Type,
    pub permissions: Permissions,
    pub xattrs:      bool,
}


/// The permissions encoded as octal values
#[derive(Copy, Clone)]
pub struct OctalPermissions {
    pub permissions: Permissions,
}

/// A file’s number of hard links on the filesystem.
///
/// Under Unix, a file can exist on the filesystem only once but appear in
/// multiple directories. However, it’s rare (but occasionally useful!) for a
/// regular file to have a link count greater than 1, so we highlight the
/// block count specifically for this case.
#[derive(Copy, Clone)]
pub struct Links {

    /// The actual link count.
    pub count: nlink_t,

    /// Whether this file is a regular file with more than one hard link.
    pub multiple: bool,
}


/// A file’s inode. Every directory entry on a Unix filesystem has an inode,
/// including directories and links, so this is applicable to everything exa
/// can deal with.
#[derive(Copy, Clone)]
pub struct Inode(pub ino_t);


/// The number of blocks that a file takes up on the filesystem, if any.
#[derive(Copy, Clone)]
pub enum Blocks {

    /// This file has the given number of blocks.
    Some(blkcnt_t),

    /// This file isn’t of a type that can take up blocks.
    None,
}


/// The ID of the user that owns a file. This will only ever be a number;
/// looking up the username is done in the `display` module.
#[derive(Copy, Clone)]
pub struct User(pub uid_t);

/// The ID of the group that a file belongs to.
#[derive(Copy, Clone)]
pub struct Group(pub gid_t);


/// A file’s size, in bytes. This is usually formatted by the `number_prefix`
/// crate into something human-readable.
#[derive(Copy, Clone)]
pub enum Size {

    /// This file has a defined size.
    Some(u64),

    /// This file has no size, or has a size but we aren’t interested in it.
    ///
    /// Under Unix, directory entries that aren’t regular files will still
    /// have a file size. For example, a directory will just contain a list of
    /// its files as its “contents” and will be specially flagged as being a
    /// directory, rather than a file. However, seeing the “file size” of this
    /// data is rarely useful — I can’t think of a time when I’ve seen it and
    /// learnt something. So we discard it and just output “-” instead.
    ///
    /// See this answer for more: <https://unix.stackexchange.com/a/68266>
    None,

    /// This file is a block or character device, so instead of a size, print
    /// out the file’s major and minor device IDs.
    ///
    /// This is what ls does as well. Without it, the devices will just have
    /// file sizes of zero.
    DeviceIDs(DeviceIDs),
}

/// The major and minor device IDs that gets displayed for device files.
///
/// You can see what these device numbers mean:
/// - <http://www.lanana.org/docs/device-list/>
/// - <http://www.lanana.org/docs/device-list/devices-2.6+.txt>
#[derive(Copy, Clone)]
pub struct DeviceIDs {
    pub major: u8,
    pub minor: u8,
}


/// One of a file’s timestamps (created, accessed, or modified).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    pub seconds: time_t,
    pub nanoseconds: time_t,
}


/// A file’s status in a Git repository. Whether a file is in a repository or
/// not is handled by the Git module, rather than having a “null” variant in
/// this enum.
#[derive(PartialEq, Copy, Clone)]
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

    /// A file that’s ignored (that matches a line in .gitignore)
    Ignored,

    /// A file that’s updated but unmerged.
    Conflicted,
}


/// A file’s complete Git status. It’s possible to make changes to a file, add
/// it to the staging area, then make *more* changes, so we need to list each
/// file’s status for both of these.
#[derive(Copy, Clone)]
pub struct Git {
    pub staged:   GitStatus,
    pub unstaged: GitStatus,
}

impl Default for Git {

    /// Create a Git status for a file with nothing done to it.
    fn default() -> Self {
        Self {
            staged: GitStatus::NotModified,
            unstaged: GitStatus::NotModified,
        }
    }
}
