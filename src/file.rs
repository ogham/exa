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
/// once, have its file extension extracted at least once, and have its stat
/// information queried at least once, so it makes sense to do all this at the
/// start and hold on to all the information.
pub struct File<'a> {
    pub name:  String,
    pub dir:   Option<&'a Dir>,
    pub ext:   Option<String>,
    pub path:  PathBuf,
    pub stat:  fs::Metadata,
    pub xattrs: Vec<Attribute>,
    pub this:  Option<Dir>,
}

impl<'a> File<'a> {
    /// Create a new File object from the given Path, inside the given Dir, if
    /// appropriate. Paths specified directly on the command-line have no Dirs.
    ///
    /// This uses `symlink_metadata` instead of `metadata`, which doesn't
    /// follow symbolic links.
    pub fn from_path(path: &Path, parent: Option<&'a Dir>, recurse: bool) -> io::Result<File<'a>> {
        fs::symlink_metadata(path).map(|stat| File::with_stat(stat, path, parent, recurse))
    }

    /// Create a new File object from the given Stat result, and other data.
    pub fn with_stat(stat: fs::Metadata, path: &Path, parent: Option<&'a Dir>, recurse: bool) -> File<'a> {
        let filename = path_filename(path);

        // If we are recursing, then the `this` field contains a Dir object
        // that represents the current File as a directory, if it is a
        // directory. This is used for the --tree option.
        let this = if recurse && stat.is_dir() {
            Dir::readdir(path).ok()
        }
        else {
            None
        };

        File {
            path:   path.to_path_buf(),
            dir:    parent,
            stat:   stat,
            ext:    ext(&filename),
            xattrs: Attribute::llist(path).unwrap_or(Vec::new()),
            name:   filename.to_string(),
            this:   this,
        }
    }

    pub fn is_directory(&self) -> bool {
        self.stat.is_dir()
    }

    pub fn is_file(&self) -> bool {
        self.stat.is_file()
    }

    pub fn is_executable_file(&self) -> bool {
        let bit = unix::fs::USER_EXECUTE;
        self.is_file() && (self.stat.permissions().mode() & bit) == bit
    }

    pub fn is_link(&self) -> bool {
        self.stat.file_type().is_symlink()
    }

    pub fn is_pipe(&self) -> bool {
        false  // TODO: Still waiting on this one...
    }

    /// Whether this file is a dotfile or not.
    pub fn is_dotfile(&self) -> bool {
        self.name.starts_with(".")
    }

    pub fn path_prefix(&self) -> String {
        let path_bytes: Vec<Component> = self.path.components().collect();
        let mut path_prefix = String::new();

        if !path_bytes.is_empty() {
            // Use init() to add all but the last component of the
            // path to the prefix. init() panics when given an
            // empty list, hence the check.
            for component in path_bytes.init().iter() {
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
        if let Ok(stat) = fs::metadata(&target_path) {
            Ok(File {
                path:   target_path.to_path_buf(),
                dir:    self.dir,
                stat:   stat,
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

    /// This file's number of hard links as a coloured string.
    ///
    /// This is important, because a file with multiple links is uncommon,
    /// while you can come across directories and other types with multiple
    /// links much more often.
    pub fn links(&self) -> f::Links {
        let count = self.stat.as_raw().nlink();

        f::Links {
            count: count,
            multiple: self.is_file() && count > 1,
        }
    }

    pub fn inode(&self) -> f::Inode {
        f::Inode(self.stat.as_raw().ino())
    }

    pub fn blocks(&self) -> f::Blocks {
        if self.is_file() || self.is_link() {
            f::Blocks::Some(self.stat.as_raw().blocks())
        }
        else {
            f::Blocks::None
        }
    }

    pub fn user(&self) -> f::User {
        f::User(self.stat.as_raw().uid())
    }

    pub fn group(&self) -> f::Group {
        f::Group(self.stat.as_raw().gid())
    }

    /// This file's size, formatted using the given way, as a coloured string.
    ///
    /// For directories, no size is given. Although they do have a size on
    /// some filesystems, I've never looked at one of those numbers and gained
    /// any information from it, so by emitting "-" instead, the table is less
    /// cluttered with numbers.
    pub fn size(&self) -> f::Size {
        if self.is_directory() {
            f::Size::None
        }
        else {
            f::Size::Some(self.stat.len())
        }
    }

    pub fn timestamp(&self, time_type: TimeType) -> f::Time {
        let time_in_seconds = match time_type {
            TimeType::FileAccessed => self.stat.as_raw().atime(),
            TimeType::FileModified => self.stat.as_raw().mtime(),
            TimeType::FileCreated  => self.stat.as_raw().ctime(),
        };

        f::Time(time_in_seconds)
    }

    /// This file's type, represented by a coloured character.
    ///
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

    pub fn permissions(&self) -> f::Permissions {
        let bits = self.stat.permissions().mode();
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

    pub fn extension_is_one_of(&self, choices: &[&str]) -> bool {
        match self.ext {
            Some(ref ext)  => choices.contains(&&ext[..]),
            None           => false,
        }
    }

    pub fn name_is_one_of(&self, choices: &[&str]) -> bool {
        choices.contains(&&self.name[..])
    }

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
fn ext<'a>(name: &'a str) -> Option<String> {
    name.rfind('.').map(|p| name[p+1..].to_ascii_lowercase())
}

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

#[cfg(broken_test)]
pub mod test {
    pub use super::*;

    pub use column::{Cell, Column};
    pub use output::details::UserLocale;

    pub use users::{User, Group};
    pub use users::mock::MockUsers;

    pub use ansi_term::Style::Plain;
    pub use ansi_term::Colour::Yellow;

    #[test]
    fn extension() {
        assert_eq!(Some("dat".to_string()), super::ext("fester.dat"))
    }

    #[test]
    fn dotfile() {
        assert_eq!(Some("vimrc".to_string()), super::ext(".vimrc"))
    }

    #[test]
    fn no_extension() {
        assert_eq!(None, super::ext("jarlsberg"))
    }

    pub fn new_file(stat: io::FileStat, path: &'static str) -> File {
        File::with_stat(stat, &Path::new(path), None, false)
    }

    pub fn dummy_stat() -> io::FileStat {
        io::FileStat {
            size: 0,
            kind: io::FileType::RegularFile,
            created: 0,
            modified: 0,
            accessed: 0,
            perm: io::USER_READ,
            unstable: io::UnstableFileStat {
                inode: 0,
                device: 0,
                rdev: 0,
                nlink: 0,
                uid: 0,
                gid: 0,
                blksize: 0,
                blocks: 0,
                flags: 0,
                gen: 0,
            }
        }
    }

    pub fn dummy_locale() -> UserLocale {
        UserLocale::default()
    }

    mod users {
        use super::*;

        #[test]
        fn named() {
            let mut stat = dummy_stat();
            stat.unstable.uid = 1000;

            let file = new_file(stat, "/hi");

            let mut users = MockUsers::with_current_uid(1000);
            users.add_user(User { uid: 1000, name: "enoch".to_string(), primary_group: 100 });

            let cell = Cell::paint(Yellow.bold(), "enoch");
            assert_eq!(cell, file.display(&Column::User, &mut users, &dummy_locale()))
        }

        #[test]
        fn unnamed() {
            let mut stat = dummy_stat();
            stat.unstable.uid = 1000;

            let file = new_file(stat, "/hi");

            let mut users = MockUsers::with_current_uid(1000);

            let cell = Cell::paint(Yellow.bold(), "1000");
            assert_eq!(cell, file.display(&Column::User, &mut users, &dummy_locale()))
        }

        #[test]
        fn different_named() {
            let mut stat = dummy_stat();
            stat.unstable.uid = 1000;

            let file = new_file(stat, "/hi");

            let mut users = MockUsers::with_current_uid(3);
            users.add_user(User { uid: 1000, name: "enoch".to_string(), primary_group: 100 });

            let cell = Cell::paint(Plain, "enoch");
            assert_eq!(cell, file.display(&Column::User, &mut users, &dummy_locale()))
        }

        #[test]
        fn different_unnamed() {
            let mut stat = dummy_stat();
            stat.unstable.uid = 1000;

            let file = new_file(stat, "/hi");

            let mut users = MockUsers::with_current_uid(3);

            let cell = Cell::paint(Plain, "1000");
            assert_eq!(cell, file.display(&Column::User, &mut users, &dummy_locale()))
        }

        #[test]
        fn overflow() {
            let mut stat = dummy_stat();
            stat.unstable.uid = 2_147_483_648;

            let file = new_file(stat, "/hi");

            let mut users = MockUsers::with_current_uid(3);

            let cell = Cell::paint(Plain, "2147483648");
            assert_eq!(cell, file.display(&Column::User, &mut users, &dummy_locale()))
        }
    }

    mod groups {
        use super::*;

        #[test]
        fn named() {
            let mut stat = dummy_stat();
            stat.unstable.gid = 100;

            let file = new_file(stat, "/hi");

            let mut users = MockUsers::with_current_uid(3);
            users.add_group(Group { gid: 100, name: "folk".to_string(), members: vec![] });

            let cell = Cell::paint(Plain, "folk");
            assert_eq!(cell, file.display(&Column::Group, &mut users, &dummy_locale()))
        }

        #[test]
        fn unnamed() {
            let mut stat = dummy_stat();
            stat.unstable.gid = 100;

            let file = new_file(stat, "/hi");

            let mut users = MockUsers::with_current_uid(3);

            let cell = Cell::paint(Plain, "100");
            assert_eq!(cell, file.display(&Column::Group, &mut users, &dummy_locale()))
        }

        #[test]
        fn primary() {
            let mut stat = dummy_stat();
            stat.unstable.gid = 100;

            let file = new_file(stat, "/hi");

            let mut users = MockUsers::with_current_uid(3);
            users.add_user(User { uid: 3, name: "eve".to_string(), primary_group: 100 });
            users.add_group(Group { gid: 100, name: "folk".to_string(), members: vec![] });

            let cell = Cell::paint(Yellow.bold(), "folk");
            assert_eq!(cell, file.display(&Column::Group, &mut users, &dummy_locale()))
        }

        #[test]
        fn secondary() {
            let mut stat = dummy_stat();
            stat.unstable.gid = 100;

            let file = new_file(stat, "/hi");

            let mut users = MockUsers::with_current_uid(3);
            users.add_user(User { uid: 3, name: "eve".to_string(), primary_group: 12 });
            users.add_group(Group { gid: 100, name: "folk".to_string(), members: vec![ "eve".to_string() ] });

            let cell = Cell::paint(Yellow.bold(), "folk");
            assert_eq!(cell, file.display(&Column::Group, &mut users, &dummy_locale()))
        }

        #[test]
        fn overflow() {
            let mut stat = dummy_stat();
            stat.unstable.gid = 2_147_483_648;

            let file = new_file(stat, "/hi");

            let mut users = MockUsers::with_current_uid(3);

            let cell = Cell::paint(Plain, "2147483648");
            assert_eq!(cell, file.display(&Column::Group, &mut users, &dummy_locale()))
        }
    }
}
