//! Files, and methods and fields to access their metadata.

use std::ascii::AsciiExt;
use std::env::current_dir;
use std::fs;
use std::io::Error as IOError;
use std::io::Result as IOResult;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use fs::dir::Dir;
use fs::fields as f;

#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::os::unix::fs::FileTypeExt;

/// Constant table copied from https://doc.rust-lang.org/src/std/sys/unix/ext/fs.rs.html#11-259
/// which is currently unstable and lacks vision for stabilization,
/// see https://github.com/rust-lang/rust/issues/27712
#[allow(dead_code, non_camel_case_types)]
mod modes {
    pub type mode_t = u32;

    pub const USER_READ: mode_t = 0o400;
    pub const USER_WRITE: mode_t = 0o200;
    pub const USER_EXECUTE: mode_t = 0o100;
    pub const USER_RWX: mode_t = 0o700;
    pub const GROUP_READ: mode_t = 0o040;
    pub const GROUP_WRITE: mode_t = 0o020;
    pub const GROUP_EXECUTE: mode_t = 0o010;
    pub const GROUP_RWX: mode_t = 0o070;
    pub const OTHER_READ: mode_t = 0o004;
    pub const OTHER_WRITE: mode_t = 0o002;
    pub const OTHER_EXECUTE: mode_t = 0o001;
    pub const OTHER_RWX: mode_t = 0o007;
    pub const ALL_READ: mode_t = 0o444;
    pub const ALL_WRITE: mode_t = 0o222;
    pub const ALL_EXECUTE: mode_t = 0o111;
    pub const ALL_RWX: mode_t = 0o777;
    pub const SETUID: mode_t = 0o4000;
    pub const SETGID: mode_t = 0o2000;
    pub const STICKY_BIT: mode_t = 0o1000;
}


/// A **File** is a wrapper around one of Rust's Path objects, along with
/// associated data about the file.
///
/// Each file is definitely going to have its filename displayed at least
/// once, have its file extension extracted at least once, and have its metadata
/// information queried at least once, so it makes sense to do all this at the
/// start and hold on to all the information.
pub struct File<'dir> {

    /// The filename portion of this file's path, including the extension.
    ///
    /// This is used to compare against certain filenames (such as checking if
    /// it’s “Makefile” or something) and to highlight only the filename in
    /// colour when displaying the path.
    pub name: String,

    /// The file’s name’s extension, if present, extracted from the name.
    ///
    /// This is queried many times over, so it’s worth caching it.
    pub ext: Option<String>,

    /// The path that begat this file.
    ///
    /// Even though the file's name is extracted, the path needs to be kept
    /// around, as certain operations involve looking up the file's absolute
    /// location (such as the Git status, or searching for compiled files).
    pub path: PathBuf,

    /// A cached `metadata` call for this file.
    ///
    /// This too is queried multiple times, and is *not* cached by the OS, as
    /// it could easily change between invocations - but exa is so short-lived
    /// it's better to just cache it.
    pub metadata: fs::Metadata,

    /// A reference to the directory that contains this file, if present.
    ///
    /// Filenames that get passed in on the command-line directly will have no
    /// parent directory reference - although they technically have one on the
    /// filesystem, we'll never need to look at it, so it'll be `None`.
    /// However, *directories* that get passed in will produce files that
    /// contain a reference to it, which is used in certain operations (such
    /// as looking up a file's Git status).
    pub dir: Option<&'dir Dir>,
}

impl<'dir> File<'dir> {

    /// Create a new `File` object from the given `Path`, inside the given
    /// `Dir`, if appropriate.
    ///
    /// This uses `symlink_metadata` instead of `metadata`, which doesn't
    /// follow symbolic links.
    pub fn from_path(path: &Path, parent: Option<&'dir Dir>) -> IOResult<File<'dir>> {
        fs::symlink_metadata(path).map(|metadata| File::with_metadata(metadata, path, parent))
    }

    /// Create a new File object from the given metadata result, and other data.
    pub fn with_metadata(metadata: fs::Metadata, path: &Path, parent: Option<&'dir Dir>) -> File<'dir> {
        let filename = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None       => String::new(),
        };

        File {
            path:      path.to_path_buf(),
            dir:       parent,
            metadata:  metadata,
            ext:       ext(path),
            name:      filename,
        }
    }

    /// Whether this file is a directory on the filesystem.
    pub fn is_directory(&self) -> bool {
        self.metadata.is_dir()
    }

    /// If this file is a directory on the filesystem, then clone its
    /// `PathBuf` for use in one of our own `Dir` objects, and read a list of
    /// its contents.
    ///
    /// Returns an IO error upon failure, but this shouldn't be used to check
    /// if a `File` is a directory or not! For that, just use `is_directory()`.
    pub fn to_dir(&self, scan_for_git: bool) -> IOResult<Dir> {
        Dir::read_dir(&*self.path, scan_for_git)
    }

    /// Whether this file is a regular file on the filesystem - that is, not a
    /// directory, a link, or anything else treated specially.
    pub fn is_file(&self) -> bool {
        self.metadata.is_file()
    }

    /// Whether this file is both a regular file *and* executable for the
    /// current user. Executable files have different semantics than
    /// executable directories, and so should be highlighted differently.
    pub fn is_executable_file(&self) -> bool {
        let bit = modes::USER_EXECUTE;
        self.is_file() && (self.metadata.permissions().mode() & bit) == bit
    }

    /// Whether this file is a symlink on the filesystem.
    pub fn is_link(&self) -> bool {
        self.metadata.file_type().is_symlink()
    }

    /// Whether this file is a dotfile, based on its name. In Unix, file names
    /// beginning with a dot represent system or configuration files, and
    /// should be hidden by default.
    pub fn is_dotfile(&self) -> bool {
        self.name.starts_with('.')
    }

    /// Assuming the current file is a symlink, follows the link and
    /// returns a File object from the path the link points to.
    ///
    /// If statting the file fails (usually because the file on the
    /// other end doesn't exist), returns the path to the file
    /// that should be there.
    pub fn link_target(&self) -> FileTarget<'dir> {
        let path = match fs::read_link(&self.path) {
            Ok(path)  => path,
            Err(e)    => return FileTarget::Err(e),
        };

        let target_path = match self.dir {
            Some(dir)  => dir.join(&*path),
            None       => path
        };

        let filename = match target_path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None       => String::new(),
        };

        // Use plain `metadata` instead of `symlink_metadata` - we *want* to follow links.
        if let Ok(metadata) = fs::metadata(&target_path) {
            FileTarget::Ok(File {
                path:      target_path.to_path_buf(),
                dir:       self.dir,
                metadata:  metadata,
                ext:       ext(&target_path),
                name:      filename,
            })
        }
        else {
            FileTarget::Broken(target_path)
        }
    }

    /// This file's number of hard links.
    ///
    /// It also reports whether this is both a regular file, and a file with
    /// multiple links. This is important, because a file with multiple links
    /// is uncommon, while you can come across directories and other types
    /// with multiple links much more often. Thus, it should get highlighted
    /// more attentively.
    pub fn links(&self) -> f::Links {
        let count = self.metadata.nlink();

        f::Links {
            count: count,
            multiple: self.is_file() && count > 1,
        }
    }

    /// This file's inode.
    pub fn inode(&self) -> f::Inode {
        f::Inode(self.metadata.ino())
    }

    /// This file's number of filesystem blocks.
    ///
    /// (Not the size of each block, which we don't actually report on)
    pub fn blocks(&self) -> f::Blocks {
        if self.is_file() || self.is_link() {
            f::Blocks::Some(self.metadata.blocks())
        }
        else {
            f::Blocks::None
        }
    }

    /// The ID of the user that own this file.
    pub fn user(&self) -> f::User {
        f::User(self.metadata.uid())
    }

    /// The ID of the group that owns this file.
    pub fn group(&self) -> f::Group {
        f::Group(self.metadata.gid())
    }

    /// This file's size, if it's a regular file.
    ///
    /// For directories, no size is given. Although they do have a size on
    /// some filesystems, I've never looked at one of those numbers and gained
    /// any information from it. So it's going to be hidden instead.
    pub fn size(&self) -> f::Size {
        if self.is_directory() {
            f::Size::None
        }
        else {
            f::Size::Some(self.metadata.len())
        }
    }

    pub fn modified_time(&self) -> f::Time {
        f::Time(self.metadata.mtime())
    }

    pub fn created_time(&self) -> f::Time {
        f::Time(self.metadata.ctime())
    }

    pub fn accessed_time(&self) -> f::Time {
        f::Time(self.metadata.mtime())
    }

    /// This file's 'type'.
    ///
    /// This is used in the leftmost column of the permissions column.
    /// Although the file type can usually be guessed from the colour of the
    /// file, `ls` puts this character there, so people will expect it.
    pub fn type_char(&self) -> f::Type {
        if self.is_file() {
            f::Type::File
        }
        else if self.is_directory() {
            f::Type::Directory
        }
        else if self.is_pipe() {
            f::Type::Pipe
        }
        else if self.is_link() {
            f::Type::Link
        }
        else if self.is_char_device() {
            f::Type::CharDevice
        }
        else if self.is_block_device() {
            f::Type::BlockDevice
        }
        else if self.is_socket() {
            f::Type::Socket
        }
        else {
            f::Type::Special
        }
    }

    /// This file's permissions, with flags for each bit.
    ///
    /// The extended-attribute '@' character that you see in here is in fact
    /// added in later, to avoid querying the extended attributes more than
    /// once. (Yes, it's a little hacky.)
    pub fn permissions(&self) -> f::Permissions {
        let bits = self.metadata.permissions().mode();
        let has_bit = |bit| { bits & bit == bit };

        f::Permissions {
            user_read:      has_bit(modes::USER_READ),
            user_write:     has_bit(modes::USER_WRITE),
            user_execute:   has_bit(modes::USER_EXECUTE),
            group_read:     has_bit(modes::GROUP_READ),
            group_write:    has_bit(modes::GROUP_WRITE),
            group_execute:  has_bit(modes::GROUP_EXECUTE),
            other_read:     has_bit(modes::OTHER_READ),
            other_write:    has_bit(modes::OTHER_WRITE),
            other_execute:  has_bit(modes::OTHER_EXECUTE),
        }
    }

    /// Whether this file's extension is any of the strings that get passed in.
    ///
    /// This will always return `false` if the file has no extension.
    pub fn extension_is_one_of(&self, choices: &[&str]) -> bool {
        match self.ext {
            Some(ref ext)  => choices.contains(&&ext[..]),
            None           => false,
        }
    }

    /// Whether this file's name, including extension, is any of the strings
    /// that get passed in.
    pub fn name_is_one_of(&self, choices: &[&str]) -> bool {
        choices.contains(&&self.name[..])
    }

    /// This file's Git status as two flags: one for staged changes, and the
    /// other for unstaged changes.
    ///
    /// This requires looking at the `git` field of this file's parent
    /// directory, so will not work if this file has just been passed in on
    /// the command line.
    pub fn git_status(&self) -> f::Git {
        match self.dir {
            None    => f::Git { staged: f::GitStatus::NotModified, unstaged: f::GitStatus::NotModified },
            Some(d) => {
                let cwd = match current_dir() {
                    Err(_)  => Path::new(".").join(&self.path),
                    Ok(dir) => dir.join(&self.path),
                };

                d.git_status(&cwd, self.is_directory())
            },
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl<'dir> File<'dir> {
    /// Whether this file is a named pipe on the filesystem.
    pub fn is_pipe(&self) -> bool {
        self.metadata.file_type().is_fifo()
    }

    /// Whether this file is a char device on the filesystem.
    pub fn is_char_device(&self) -> bool {
        self.metadata.file_type().is_char_device()
    }

    /// Whether this file is a block device on the filesystem.
    pub fn is_block_device(&self) -> bool {
        self.metadata.file_type().is_block_device()
    }

    /// Whether this file is a socket on the filesystem.
    pub fn is_socket(&self) -> bool {
        self.metadata.file_type().is_socket()
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
impl<'dir> File<'dir> {
    /// Whether this file is a named pipe on the filesystem.
    pub fn is_pipe(&self) -> bool {
        false
    }

    /// Whether this file is a char device on the filesystem.
    pub fn is_char_device(&self) -> bool {
        false
    }

    /// Whether this file is a block device on the filesystem.
    pub fn is_block_device(&self) -> bool {
        false
    }

    /// Whether this file is a socket on the filesystem.
    pub fn is_socket(&self) -> bool {
        false
    }
}


impl<'a> AsRef<File<'a>> for File<'a> {
    fn as_ref(&self) -> &File<'a> {
        self
    }
}


/// Extract an extension from a file path, if one is present, in lowercase.
///
/// The extension is the series of characters after the last dot. This
/// deliberately counts dotfiles, so the ".git" folder has the extension "git".
///
/// ASCII lowercasing is used because these extensions are only compared
/// against a pre-compiled list of extensions which are known to only exist
/// within ASCII, so it's alright.
fn ext(path: &Path) -> Option<String> {
    let name = match path.file_name() {
        Some(f) => f.to_string_lossy().to_string(),
        None => return None,
    };

    name.rfind('.').map(|p| name[p+1..].to_ascii_lowercase())
}


/// The result of following a symlink.
pub enum FileTarget<'dir> {

    /// The symlink pointed at a file that exists.
    Ok(File<'dir>),

    /// The symlink pointed at a file that does not exist. Holds the path
    /// where the file would be, if it existed.
    Broken(PathBuf),

    /// There was an IO error when following the link. This can happen if the
    /// file isn't a link to begin with, but also if, say, we don't have
    /// permission to follow it.
    Err(IOError),
}


#[cfg(test)]
mod test {
    use super::ext;
    use std::path::Path;

    #[test]
    fn extension() {
        assert_eq!(Some("dat".to_string()), ext(Path::new("fester.dat")))
    }

    #[test]
    fn dotfile() {
        assert_eq!(Some("vimrc".to_string()), ext(Path::new(".vimrc")))
    }

    #[test]
    fn no_extension() {
        assert_eq!(None, ext(Path::new("jarlsberg")))
    }
}
