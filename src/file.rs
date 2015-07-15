//! Files, and methods and fields to access their metadata.

use std::ascii::AsciiExt;
use std::env::current_dir;
use std::fs;
use std::io;
use std::os::unix;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};

use unicode_width::UnicodeWidthStr;

use dir::Dir;
use options::TimeType;
use feature::Attribute;

use self::fields as f;


/// A **File** is a wrapper around one of Rust's Path objects, along with
/// associated data about the file.
///
/// Each file is definitely going to have its filename displayed at least
/// once, have its file extension extracted at least once, and have its metadata
/// information queried at least once, so it makes sense to do all this at the
/// start and hold on to all the information.
pub struct File<'dir> {

    /// This file's name, as a UTF-8 encoded String.
    pub name:  String,

    /// The file's name's extension, if present, extracted from the name. This
    /// is queried a lot, so it's worth being cached.
    pub ext:   Option<String>,

    /// The path that begat this file. Even though the file's name is
    /// extracted, the path needs to be kept around, as certain operations
    /// involve looking up the file's absolute location (such as the Git
    /// status, or searching for compiled files).
    pub path:  PathBuf,

    /// A cached `metadata` call for this file. This is queried multiple
    /// times, and is *not* cached by the OS, as it could easily change
    /// between invocations - but exa is so short-lived it's better to just
    /// cache it.
    pub metadata:  fs::Metadata,

    /// List of this file's extended attributes. These are only loaded if the
    /// `xattr` feature is in use.
    pub xattrs: Vec<Attribute>,

    /// A reference to the directory that contains this file, if present.
    ///
    /// Filenames that get passed in on the command-line directly will have no
    /// parent directory reference - although they technically have one on the
    /// filesystem, we'll never need to look at it, so it'll be `None`.
    /// However, *directories* that get passed in will produce files that
    /// contain a reference to it, which is used in certain operations (such
    /// as looking up a file's Git status).
    pub dir:   Option<&'dir Dir>,

    /// If this `File` is also a directory, then this field is the same file
    /// as a `Dir`.
    pub this:  Option<Dir>,
}

impl<'dir> File<'dir> {
    /// Create a new `File` object from the given `Path`, inside the given
    /// `Dir`, if appropriate.
    ///
    /// This uses `symlink_metadata` instead of `metadata`, which doesn't
    /// follow symbolic links.
    pub fn from_path(path: &Path, parent: Option<&'dir Dir>, recurse: bool) -> io::Result<File<'dir>> {
        fs::symlink_metadata(path).map(|metadata| File::with_metadata(metadata, path, parent, recurse))
    }

    /// Create a new File object from the given metadata result, and other data.
    pub fn with_metadata(metadata: fs::Metadata, path: &Path, parent: Option<&'dir Dir>, recurse: bool) -> File<'dir> {
        let filename = path_filename(path);

        // If we are recursing, then the `this` field contains a Dir object
        // that represents the current File as a directory, if it is a
        // directory. This is used for the --tree option.
        let this = if recurse && metadata.is_dir() {
            Dir::readdir(path).ok()
        }
        else {
            None
        };

        File {
            path:   path.to_path_buf(),
            dir:    parent,
            metadata:   metadata,
            ext:    ext(&filename),
            xattrs: Attribute::llist(path).unwrap_or(Vec::new()),
            name:   filename.to_string(),
            this:   this,
        }
    }

    /// Whether this file is a directory on the filesystem.
    pub fn is_directory(&self) -> bool {
        self.metadata.is_dir()
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
        let bit = unix::fs::USER_EXECUTE;
        self.is_file() && (self.metadata.permissions().mode() & bit) == bit
    }

    /// Whether this file is a symlink on the filesystem.
    pub fn is_link(&self) -> bool {
        self.metadata.file_type().is_symlink()
    }

    /// Whether this file is a named pipe on the filesystem.
    pub fn is_pipe(&self) -> bool {
        false  // TODO: Still waiting on this one...
    }

    /// Whether this file is a dotfile, based on its name. In Unix, file names
    /// beginning with a dot represent system or configuration files, and
    /// should be hidden by default.
    pub fn is_dotfile(&self) -> bool {
        self.name.starts_with(".")
    }

    /// Constructs the 'path prefix' of this file, which is the portion of the
    /// path up to, but not including, the file name.
    ///
    /// This gets used when displaying the path a symlink points to. In
    /// certain cases, it may return an empty-length string. Examples:
    ///
    /// - `code/exa/file.rs` has `code/exa/` as its prefix, including the
    ///   trailing slash.
    /// - `code/exa` has just `code/` as its prefix.
    /// - `code` has the empty string as its prefix.
    /// - `/` also has the empty string as its prefix. It does not have a
    ///   trailing slash, as the slash constitutes the 'name' of this file.
    pub fn path_prefix(&self) -> String {
        let components: Vec<Component> = self.path.components().collect();
        let mut path_prefix = String::new();

        if let Some((_, components_init)) = components.split_last() {
            for component in components_init.iter() {
                path_prefix.push_str(&*component.as_os_str().to_string_lossy());

                if component != &Component::RootDir {
                    path_prefix.push_str("/");
                }
            }
        }

        path_prefix
    }

    /// The Unicode 'display width' of the filename.
    ///
    /// This is related to the number of graphemes in the string: most
    /// characters are 1 columns wide, but in some contexts, certain
    /// characters are actually 2 columns wide.
    pub fn file_name_width(&self) -> usize {
        UnicodeWidthStr::width(&self.name[..])
    }

    /// Assuming the current file is a symlink, follows the link and
    /// returns a File object from the path the link points to.
    ///
    /// If statting the file fails (usually because the file on the
    /// other end doesn't exist), returns the *filename* of the file
    /// that should be there.
    pub fn link_target(&self) -> Result<File, String> {
        let path = match fs::read_link(&self.path) {
            Ok(path)  => path,
            Err(_)    => return Err(self.name.clone()),
        };

        let target_path = match self.dir {
            Some(dir)  => dir.join(&*path),
            None       => path
        };

        let filename = path_filename(&target_path);

        // Use plain `metadata` instead of `symlink_metadata` - we *want* to follow links.
        if let Ok(metadata) = fs::metadata(&target_path) {
            Ok(File {
                path:   target_path.to_path_buf(),
                dir:    self.dir,
                metadata:   metadata,
                ext:    ext(&filename),
                xattrs: Attribute::list(&target_path).unwrap_or(Vec::new()),
                name:   filename.to_string(),
                this:   None,
            })
        }
        else {
            Err(filename.to_string())
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

    /// One of this file's timestamps, as a number in seconds.
    pub fn timestamp(&self, time_type: TimeType) -> f::Time {
        let time_in_seconds = match time_type {
            TimeType::FileAccessed => self.metadata.atime(),
            TimeType::FileModified => self.metadata.mtime(),
            TimeType::FileCreated  => self.metadata.ctime(),
        };

        f::Time(time_in_seconds)
    }

    /// This file's 'type'.
    ///
    /// This is used in the leftmost column of the permissions column.
    /// Although the file type can usually be guessed from the colour of the
    /// file, `ls` puts this character there, so people will expect it.
    fn type_char(&self) -> f::Type {
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
        else {
            f::Type::Special
        }
    }

    /// This file's permissions, with flags for each bit.
    pub fn permissions(&self) -> f::Permissions {
        let bits = self.metadata.permissions().mode();
        let has_bit = |bit| { bits & bit == bit };

        f::Permissions {
            file_type:      self.type_char(),
            user_read:      has_bit(unix::fs::USER_READ),
            user_write:     has_bit(unix::fs::USER_WRITE),
            user_execute:   has_bit(unix::fs::USER_EXECUTE),
            group_read:     has_bit(unix::fs::GROUP_READ),
            group_write:    has_bit(unix::fs::GROUP_WRITE),
            group_execute:  has_bit(unix::fs::GROUP_EXECUTE),
            other_read:     has_bit(unix::fs::OTHER_READ),
            other_write:    has_bit(unix::fs::OTHER_WRITE),
            other_execute:  has_bit(unix::fs::OTHER_EXECUTE),
            attribute:      !self.xattrs.is_empty()
        }
    }

    /// For this file, return a vector of alternate file paths that, if any of
    /// them exist, mean that *this* file should be coloured as `Compiled`.
    ///
    /// The point of this is to highlight compiled files such as `foo.o` when
    /// their source file `foo.c` exists in the same directory. It's too
    /// dangerous to highlight *all* compiled, so the paths in this vector
    /// are checked for existence first: for example, `foo.js` is perfectly
    /// valid without `foo.coffee`.
    pub fn get_source_files(&self) -> Vec<PathBuf> {
        if let Some(ref ext) = self.ext {
            match &ext[..] {
                "class" => vec![self.path.with_extension("java")],  // Java
                "css"   => vec![self.path.with_extension("sass"),   self.path.with_extension("less")],  // SASS, Less
                "elc"   => vec![self.path.with_extension("el")],    // Emacs Lisp
                "hi"    => vec![self.path.with_extension("hs")],    // Haskell
                "js"    => vec![self.path.with_extension("coffee"), self.path.with_extension("ts")],  // CoffeeScript, TypeScript
                "o"     => vec![self.path.with_extension("c"),      self.path.with_extension("cpp")], // C, C++
                "pyc"   => vec![self.path.with_extension("py")],    // Python

                "aux" => vec![self.path.with_extension("tex")],  // TeX: auxiliary file
                "bbl" => vec![self.path.with_extension("tex")],  // BibTeX bibliography file
                "blg" => vec![self.path.with_extension("tex")],  // BibTeX log file
                "lof" => vec![self.path.with_extension("tex")],  // TeX list of figures
                "log" => vec![self.path.with_extension("tex")],  // TeX log file
                "lot" => vec![self.path.with_extension("tex")],  // TeX list of tables
                "toc" => vec![self.path.with_extension("tex")],  // TeX table of contents

                _ => vec![],  // No source files if none of the above
            }
        }
        else {
            vec![]  // No source files if there's no extension, either!
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

/// Extract the filename to display from a path, converting it from UTF-8
/// lossily, into a String.
///
/// The filename to display is the last component of the path. However,
/// the path has no components for `.`, `..`, and `/`, so in these
/// cases, the entire path is used.
fn path_filename(path: &Path) -> String {
    match path.iter().last() {
        Some(os_str) => os_str.to_string_lossy().to_string(),
        None => ".".to_string(),  // can this even be reached?
    }
}

/// Extract an extension from a string, if one is present, in lowercase.
///
/// The extension is the series of characters after the last dot. This
/// deliberately counts dotfiles, so the ".git" folder has the extension "git".
///
/// ASCII lowercasing is used because these extensions are only compared
/// against a pre-compiled list of extensions which are known to only exist
/// within ASCII, so it's alright.
fn ext(name: &str) -> Option<String> {
    name.rfind('.').map(|p| name[p+1..].to_ascii_lowercase())
}


/// Wrapper types for the values returned from `File` objects.
///
/// The methods of `File` don't return formatted strings; neither do they
/// return raw numbers representing timestamps or user IDs. Instead, they will
/// return an object in this `fields` module. These objects are later rendered
/// into formatted strings in the `output/details` module.
pub mod fields {
    use std::os::unix::raw::{blkcnt_t, gid_t, ino_t, nlink_t, time_t, uid_t};

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
        pub attribute:      bool,
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
}


#[cfg(test)]
mod test {
    use super::ext;

    #[test]
    fn extension() {
        assert_eq!(Some("dat".to_string()), ext("fester.dat"))
    }

    #[test]
    fn dotfile() {
        assert_eq!(Some("vimrc".to_string()), ext(".vimrc"))
    }

    #[test]
    fn no_extension() {
        assert_eq!(None, ext("jarlsberg"))
    }
}
