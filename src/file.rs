use std::old_io::{fs, IoResult};
use std::old_io as io;
use std::ascii::AsciiExt;

use ansi_term::{ANSIString, ANSIStrings, Colour, Style};
use ansi_term::Style::Plain;
use ansi_term::Colour::{Red, Green, Yellow, Blue, Purple, Cyan, Fixed};

use users::Users;

use locale;
use output::details::UserLocale;

use number_prefix::{binary_prefix, decimal_prefix, Prefixed, Standalone, PrefixNames};

use datetime::local::{LocalDateTime, DatePiece};
use datetime::format::{DateFormat};

use column::{Column, Cell};
use column::Column::*;
use dir::Dir;
use filetype::HasType;
use options::{SizeFormat, TimeType};

/// This grey value is directly in between white and black, so it's guaranteed
/// to show up on either backgrounded terminal.
pub static GREY: Colour = Fixed(244);

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
    pub path:  Path,
    pub stat:  io::FileStat,
    pub this:  Option<Dir>,
}

impl<'a> File<'a> {
    /// Create a new File object from the given Path, inside the given Dir, if
    /// appropriate. Paths specified directly on the command-line have no Dirs.
    ///
    /// This uses lstat instead of stat, which doesn't follow symbolic links.
    pub fn from_path(path: &Path, parent: Option<&'a Dir>, recurse: bool) -> IoResult<File<'a>> {
        fs::lstat(path).map(|stat| File::with_stat(stat, path, parent, recurse))
    }

    /// Create a new File object from the given Stat result, and other data.
    pub fn with_stat(stat: io::FileStat, path: &Path, parent: Option<&'a Dir>, recurse: bool) -> File<'a> {

        // The filename to display is the last component of the path. However,
        // the path has no components for `.`, `..`, and `/`, so in these
        // cases, the entire path is used.
        let bytes = match path.components().last() {
            Some(b) => b,
            None => path.as_vec(),
        };

        // Convert the string to UTF-8, replacing any invalid characters with
        // replacement characters.
        let filename = String::from_utf8_lossy(bytes);

        // If we are recursing, then the `this` field contains a Dir object
        // that represents the current File as a directory, if it is a
        // directory. This is used for the --tree option.
        let this = if recurse && stat.kind == io::FileType::Directory {
            Dir::readdir(path).ok()
        }
        else {
            None
        };

        File {
            path:  path.clone(),
            dir:   parent,
            stat:  stat,
            name:  filename.to_string(),
            ext:   ext(&filename),
            this:  this,
        }
    }

    /// Whether this file is a dotfile or not.
    pub fn is_dotfile(&self) -> bool {
        self.name.starts_with(".")
    }

    /// Whether this file is a temporary file or not.
    pub fn is_tmpfile(&self) -> bool {
        let name = &self.name;
        name.ends_with("~") || (name.starts_with("#") && name.ends_with("#"))
    }

    /// Get the data for a column, formatted as a coloured string.
    pub fn display<U: Users>(&self, column: &Column, users_cache: &mut U, locale: &UserLocale) -> Cell {
        match *column {
            Permissions     => self.permissions_string(),
            FileSize(f)     => self.file_size(f, &locale.numeric),
            Timestamp(t, y) => self.timestamp(t, y, &locale.time),
            HardLinks       => self.hard_links(&locale.numeric),
            Inode           => self.inode(),
            Blocks          => self.blocks(&locale.numeric),
            User            => self.user(users_cache),
            Group           => self.group(users_cache),
            GitStatus       => self.git_status(),
        }
    }

    /// The "file name view" is what's displayed in the column and lines
    /// views, but *not* in the grid view.
    ///
    /// It consists of the file name coloured in the appropriate style,
    /// with special formatting for a symlink.
    pub fn file_name_view(&self) -> String {
        if self.stat.kind == io::FileType::Symlink {
            self.symlink_file_name_view()
        }
        else {
            self.file_colour().paint(&*self.name).to_string()
        }
    }

    /// If this file is a symlink, returns a string displaying its name,
    /// and an arrow pointing to the file it links to, which is also
    /// coloured in the appropriate style.
    ///
    /// If the symlink target doesn't exist, then instead of displaying
    /// an error, highlight the target and arrow in red. The error would
    /// be shown out of context, and it's almost always because the
    /// target doesn't exist.
    fn symlink_file_name_view(&self) -> String {
        let name = &*self.name;
        let style = self.file_colour();

        if let Ok(path) = fs::readlink(&self.path) {
            let target_path = match self.dir {
                Some(dir) => dir.join(path),
                None => path,
            };

            match self.target_file(&target_path) {
                Ok(file) => format!("{} {} {}{}{}",
                                   style.paint(name),
                                   GREY.paint("=>"),
                                   Cyan.paint(target_path.dirname_str().unwrap()),
                                   Cyan.paint("/"),
                                   file.file_colour().paint(&file.name)),
                Err(filename) => format!("{} {} {}",
                                         style.paint(name),
                                         Red.paint("=>"),
                                         Red.underline().paint(&filename)),
            }
        }
        else {
            style.paint(name).to_string()
        }
    }

    /// The `ansi_term::Style` that this file's name should be painted.
    pub fn file_colour(&self) -> Style {
        self.get_type().style()
    }

    /// The Unicode 'display width' of the filename.
    ///
    /// This is related to the number of graphemes in the string: most
    /// characters are 1 columns wide, but in some contexts, certain
    /// characters are actually 2 columns wide.
    pub fn file_name_width(&self) -> usize {
        self.name.width(false)
    }

    /// Assuming the current file is a symlink, follows the link and
    /// returns a File object from the path the link points to.
    ///
    /// If statting the file fails (usually because the file on the
    /// other end doesn't exist), returns the *filename* of the file
    /// that should be there.
    fn target_file(&self, target_path: &Path) -> Result<File, String> {
        let v = target_path.filename().unwrap();
        let filename = String::from_utf8_lossy(v);

        // Use stat instead of lstat - we *want* to follow links.
        if let Ok(stat) = fs::stat(target_path) {
            Ok(File {
                path:  target_path.clone(),
                dir:   self.dir,
                stat:  stat,
                name:  filename.to_string(),
                ext:   ext(&filename),
                this:  None,
            })
        }
        else {
            Err(filename.to_string())
        }
    }

    /// This file's number of hard links as a coloured string.
    fn hard_links(&self, locale: &locale::Numeric) -> Cell {
        let style = if self.has_multiple_links() { Red.on(Yellow) } else { Red.normal() };
        Cell::paint(style, &locale.format_int(self.stat.unstable.nlink as isize)[..])
    }

    /// Whether this is a regular file with more than one link.
    ///
    /// This is important, because a file with multiple links is uncommon,
    /// while you can come across directories and other types with multiple
    /// links much more often.
    fn has_multiple_links(&self) -> bool {
        self.stat.kind == io::FileType::RegularFile && self.stat.unstable.nlink > 1
    }

    /// This file's inode as a coloured string.
    fn inode(&self) -> Cell {
        Cell::paint(Purple.normal(), &*self.stat.unstable.inode.to_string())
    }

    /// This file's number of filesystem blocks (if available) as a coloured string.
    fn blocks(&self, locale: &locale::Numeric) -> Cell {
        if self.stat.kind == io::FileType::RegularFile || self.stat.kind == io::FileType::Symlink {
            Cell::paint(Cyan.normal(), &locale.format_int(self.stat.unstable.blocks as isize)[..])
        }
        else {
            Cell { text: GREY.paint("-").to_string(), length: 1 }
        }
    }

    /// This file's owner's username as a coloured string.
    ///
    /// If the user is not present, then it formats the uid as a number
    /// instead. This usually happens when a user is deleted, but still owns
    /// files.
    fn user<U: Users>(&self, users_cache: &mut U) -> Cell {
        let uid = self.stat.unstable.uid as i32;

        let user_name = match users_cache.get_user_by_uid(uid) {
            Some(user) => user.name,
            None => uid.to_string(),
        };

        let style = if users_cache.get_current_uid() == uid { Yellow.bold() } else { Plain };
        Cell::paint(style, &*user_name)
    }

    /// This file's group name as a coloured string.
    ///
    /// As above, if not present, it formats the gid as a number instead.
    fn group<U: Users>(&self, users_cache: &mut U) -> Cell {
        let gid = self.stat.unstable.gid as u32;
        let mut style = Plain;

        let group_name = match users_cache.get_group_by_gid(gid) {
            Some(group) => {
                let current_uid = users_cache.get_current_uid();
                if let Some(current_user) = users_cache.get_user_by_uid(current_uid) {
                    if current_user.primary_group == group.gid || group.members.contains(&current_user.name) {
                        style = Yellow.bold();
                    }
                }
                group.name
            },
            None => gid.to_string(),
        };

        Cell::paint(style, &*group_name)
    }

    /// This file's size, formatted using the given way, as a coloured string.
    ///
    /// For directories, no size is given. Although they do have a size on
    /// some filesystems, I've never looked at one of those numbers and gained
    /// any information from it, so by emitting "-" instead, the table is less
    /// cluttered with numbers.
    fn file_size(&self, size_format: SizeFormat, locale: &locale::Numeric) -> Cell {
        if self.stat.kind == io::FileType::Directory {
            Cell { text: GREY.paint("-").to_string(), length: 1 }
        }
        else {
            let result = match size_format {
                SizeFormat::DecimalBytes => decimal_prefix(self.stat.size as f64),
                SizeFormat::BinaryBytes  => binary_prefix(self.stat.size as f64),
                SizeFormat::JustBytes    => return Cell::paint(Green.bold(), &locale.format_int(self.stat.size as isize)[..]),
            };

            match result {
                Standalone(bytes) => Cell::paint(Green.bold(), &*bytes.to_string()),
                Prefixed(prefix, n) => {
                    let number = if n < 10f64 { locale.format_float(n, 1) } else { locale.format_int(n as isize) };
                    let symbol = prefix.symbol();

                    Cell {
                        text: ANSIStrings( &[ Green.bold().paint(&number[..]), Green.paint(symbol) ]).to_string(),
                        length: number.len() + symbol.len(),
                    }
                }
            }
        }
    }

    fn timestamp(&self, time_type: TimeType, current_year: i64, locale: &locale::Time) -> Cell {

        // Need to convert these values from milliseconds into seconds.
        let time_in_seconds = match time_type {
            TimeType::FileAccessed => self.stat.accessed,
            TimeType::FileModified => self.stat.modified,
            TimeType::FileCreated  => self.stat.created,
        } as i64 / 1000;

        let date = LocalDateTime::at(time_in_seconds);

        let format = if date.year() == current_year {
                DateFormat::parse("{2>:D} {:M} {2>:h}:{02>:m}").unwrap()
            }
            else {
                DateFormat::parse("{2>:D} {:M} {5>:Y}").unwrap()
            };

        Cell::paint(Blue.normal(), &format.format(date, locale))
    }

    /// This file's type, represented by a coloured character.
    ///
    /// Although the file type can usually be guessed from the colour of the
    /// file, `ls` puts this character there, so people will expect it.
    fn type_char(&self) -> ANSIString {
        return match self.stat.kind {
            io::FileType::RegularFile   => Plain.paint("."),
            io::FileType::Directory     => Blue.paint("d"),
            io::FileType::NamedPipe     => Yellow.paint("|"),
            io::FileType::BlockSpecial  => Purple.paint("s"),
            io::FileType::Symlink       => Cyan.paint("l"),
            io::FileType::Unknown       => Plain.paint("?"),
        }
    }

    /// Generate the "rwxrwxrwx" permissions string, like how ls does it.
    ///
    /// Each character is given its own colour. The first three permission
    /// bits are bold because they're the ones used most often, and executable
    /// files are underlined to make them stand out more.
    fn permissions_string(&self) -> Cell {
        let bits = self.stat.perm;
        let executable_colour = match self.stat.kind {
            io::FileType::RegularFile => Green.bold().underline(),
            _ => Green.bold(),
        };

        let string = ANSIStrings(&[
            self.type_char(),
            File::permission_bit(&bits, io::USER_READ,     "r", Yellow.bold()),
            File::permission_bit(&bits, io::USER_WRITE,    "w", Red.bold()),
            File::permission_bit(&bits, io::USER_EXECUTE,  "x", executable_colour),
            File::permission_bit(&bits, io::GROUP_READ,    "r", Yellow.normal()),
            File::permission_bit(&bits, io::GROUP_WRITE,   "w", Red.normal()),
            File::permission_bit(&bits, io::GROUP_EXECUTE, "x", Green.normal()),
            File::permission_bit(&bits, io::OTHER_READ,    "r", Yellow.normal()),
            File::permission_bit(&bits, io::OTHER_WRITE,   "w", Red.normal()),
            File::permission_bit(&bits, io::OTHER_EXECUTE, "x", Green.normal()),
        ]).to_string();

        Cell { text: string, length: 10 }
    }

    /// Helper method for the permissions string.
    fn permission_bit(bits: &io::FilePermission, bit: io::FilePermission, character: &'static str, style: Style) -> ANSIString<'static> {
        if bits.contains(bit) {
            style.paint(character)
        }
        else {
            GREY.paint("-")
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
    pub fn get_source_files(&self) -> Vec<Path> {
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

    fn git_status(&self) -> Cell {
        let status = match self.dir {
            Some(d) => d.git_status(&self.path, self.stat.kind == io::FileType::Directory),
            None    => GREY.paint("--").to_string(),
        };

        Cell { text: status, length: 2 }
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

#[cfg(test)]
pub mod test {
    pub use super::*;
    pub use column::{Cell, Column};
    pub use std::old_io as io;
    pub use users::{User, Group};
    pub use users::mock::MockUsers;

    pub use ansi_term::Style::Plain;
    pub use ansi_term::Colour::Yellow;

    pub use output::details::UserLocale;

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
    }
}
