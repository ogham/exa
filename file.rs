use std::io::fs;
use std::io;

use colours::{Plain, Style, Black, Red, Green, Yellow, Blue, Purple, Cyan};
use column::{Column, Permissions, FileName, FileSize, User, Group};
use format::{format_metric_bytes, format_IEC_bytes};
use unix::{get_user_name, get_group_name};
use sort::SortPart;

static MEDIA_TYPES: &'static [&'static str] = &[
    "png", "jpeg", "jpg", "gif", "bmp", "tiff", "tif",
    "ppm", "pgm", "pbm", "pnm", "webp", "raw", "arw",
    "svg", "pdf", "stl", "eps", "dvi", "ps" ];

static COMPRESSED_TYPES: &'static [&'static str] = &[
    "zip", "tar", "Z", "gz", "bz2", "a", "ar", "7z",
    "iso", "dmg", "tc", "rar", "par" ];

// Instead of working with Rust's Paths, we have our own File object
// that holds the Path and various cached information. Each file is
// definitely going to have its filename used at least once, its stat
// information queried at least once, and its file extension extracted
// at least once, so we may as well carry around that information with
// the actual path.

pub struct File<'a> {
    pub name: &'a str,
    pub ext:  Option<&'a str>,
    pub path: &'a Path,
    pub stat: io::FileStat,
    pub parts: Vec<SortPart>,
}

impl<'a> File<'a> {
    pub fn from_path(path: &'a Path) -> File<'a> {
        // Getting the string from a filename fails whenever it's not
        // UTF-8 representable - just assume it is for now.
        let filename: &str = path.filename_str().unwrap();

        // Use lstat here instead of file.stat(), as it doesn't follow
        // symbolic links. Otherwise, the stat() call will fail if it
        // encounters a link that's target is non-existent.
        let stat: io::FileStat = match fs::lstat(path) {
            Ok(stat) => stat,
            Err(e) => fail!("Couldn't stat {}: {}", filename, e),
        };

        return File {
            path:  path,
            stat:  stat,
            name:  filename,
            ext:   File::ext(filename),
            parts: SortPart::split_into_parts(filename),
        };
    }

    fn ext(name: &'a str) -> Option<&'a str> {
        // The extension is the series of characters after a dot at
        // the end of a filename. This deliberately also counts
        // dotfiles - the ".git" folder has the extension "git".
        let re = regex!(r"\.([^.]+)$");
        re.captures(name).map(|caps| caps.at(1))
    }

    pub fn is_dotfile(&self) -> bool {
        self.name.starts_with(".")
    }

    pub fn display(&self, column: &Column) -> String {
        match *column {
            Permissions => self.permissions_string(),
            FileName => self.file_colour().paint(self.name),
            FileSize(use_iec) => self.file_size(use_iec),

            // Display the ID if the user/group doesn't exist, which
            // usually means it was deleted but its files weren't.
            User(uid) => {
                let style = if uid == self.stat.unstable.uid { Yellow.bold() } else { Plain };
                let string = get_user_name(self.stat.unstable.uid as i32).unwrap_or(self.stat.unstable.uid.to_str());
                return style.paint(string.as_slice());
            },
            Group => get_group_name(self.stat.unstable.gid as u32).unwrap_or(self.stat.unstable.gid.to_str()),
        }
    }

    fn file_size(&self, use_iec_prefixes: bool) -> String {
        // Don't report file sizes for directories. I've never looked
        // at one of those numbers and gained any information from it.
        if self.stat.kind == io::TypeDirectory {
            Black.bold().paint("-")
        } else {
            let (size, suffix) = if use_iec_prefixes {
                format_IEC_bytes(self.stat.size)
            } else {
                format_metric_bytes(self.stat.size)
            };

            return format!("{}{}", Green.bold().paint(size.as_slice()), Green.paint(suffix.as_slice()));
        }
    }

    fn type_char(&self) -> String {
        return match self.stat.kind {
            io::TypeFile         => ".".to_string(),
            io::TypeDirectory    => Blue.paint("d"),
            io::TypeNamedPipe    => Yellow.paint("|"),
            io::TypeBlockSpecial => Purple.paint("s"),
            io::TypeSymlink      => Cyan.paint("l"),
            _                    => "?".to_string(),
        }
    }

    fn file_colour(&self) -> Style {
        if self.stat.kind == io::TypeDirectory {
            Blue.normal()
        }
        else if self.stat.perm.contains(io::UserExecute) {
            Green.bold()
        }
        else if self.name.ends_with("~") {
            Black.bold()
        }
        else if self.name.starts_with("README") {
            Yellow.bold().underline()
        }
        else if self.ext.is_some() && MEDIA_TYPES.iter().any(|&s| s == self.ext.unwrap()) {
            Purple.normal()
        }
        else if self.ext.is_some() && COMPRESSED_TYPES.iter().any(|&s| s == self.ext.unwrap()) {
            Red.normal()
        }
        else {
            Plain
        }
    }

    fn permissions_string(&self) -> String {
        let bits = self.stat.perm;
        return format!("{}{}{}{}{}{}{}{}{}{}",
            self.type_char(),

            // The first three are bold because they're the ones used
            // most often.
            File::permission_bit(bits, io::UserRead,     "r", Yellow.bold()),
            File::permission_bit(bits, io::UserWrite,    "w", Red.bold()),
            File::permission_bit(bits, io::UserExecute,  "x", Green.bold().underline()),
            File::permission_bit(bits, io::GroupRead,    "r", Yellow.normal()),
            File::permission_bit(bits, io::GroupWrite,   "w", Red.normal()),
            File::permission_bit(bits, io::GroupExecute, "x", Green.normal()),
            File::permission_bit(bits, io::OtherRead,    "r", Yellow.normal()),
            File::permission_bit(bits, io::OtherWrite,   "w", Red.normal()),
            File::permission_bit(bits, io::OtherExecute, "x", Green.normal()),
       );
    }

    fn permission_bit(bits: io::FilePermission, bit: io::FilePermission, character: &'static str, style: Style) -> String {
        if bits.contains(bit) {
            style.paint(character.as_slice())
        } else {
            Black.bold().paint("-".as_slice())
        }
    }
}
