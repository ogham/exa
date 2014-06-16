use std::io::fs;
use std::io;

use colours::{Plain, Style, Black, Red, Green, Yellow, Blue, Purple, Cyan, Fixed};
use column::{Column, Permissions, FileName, FileSize, User, Group};
use format::{format_metric_bytes, format_IEC_bytes};
use unix::{get_user_name, get_group_name};
use sort::SortPart;
use dir::Dir;

static IMAGE_TYPES: &'static [&'static str] = &[
    "png", "jpeg", "jpg", "gif", "bmp", "tiff", "tif",
    "ppm", "pgm", "pbm", "pnm", "webp", "raw", "arw",
    "svg", "pdf", "stl", "eps", "dvi", "ps", "cbr",
    "cbz", "xpm", "ico" ];

static VIDEO_TYPES: &'static [&'static str] = &[
    "avi", "flv", "m2v", "mkv", "mov", "mp4", "mpeg",
     "mpg", "ogm", "ogv", "vob", "wmv" ];

static MUSIC_TYPES: &'static [&'static str] = &[
    "aac", "m4a", "mp3", "ogg" ];

static MUSIC_LOSSLESS: &'static [&'static str] = &[
    "alac", "ape", "flac", "wav" ];

static COMPRESSED_TYPES: &'static [&'static str] = &[
    "zip", "tar", "Z", "gz", "bz2", "a", "ar", "7z",
    "iso", "dmg", "tc", "rar", "par" ];

static DOCUMENT_TYPES: &'static [&'static str] = &[
    "djvu", "doc", "docx", "eml", "eps", "odp", "ods",
    "odt", "pdf", "ppt", "pptx", "xls", "xlsx" ];

static TEMP_TYPES: &'static [&'static str] = &[
    "tmp", "swp", "swo", "swn", "bak" ];

static CRYPTO_TYPES: &'static [&'static str] = &[
    "asc", "gpg", "sig", "signature", "pgp" ];


// Instead of working with Rust's Paths, we have our own File object
// that holds the Path and various cached information. Each file is
// definitely going to have its filename used at least once, its stat
// information queried at least once, and its file extension extracted
// at least once, so we may as well carry around that information with
// the actual path.

pub struct File<'a> {
    pub name: &'a str,
    pub dir:  &'a Dir<'a>,
    pub ext:  Option<&'a str>,
    pub path: &'a Path,
    pub stat: io::FileStat,
    pub parts: Vec<SortPart>,
}

impl<'a> File<'a> {
    pub fn from_path(path: &'a Path, parent: &'a Dir) -> File<'a> {
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
            dir:   parent,
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

    fn is_tmpfile(&self) -> bool {
        self.name.ends_with("~") || (self.name.starts_with("#") && self.name.ends_with("#"))
    }
        
    // Highlight the compiled versions of files. Some of them, like .o,
    // get special highlighting when they're alone because there's no
    // point in existing without their source. Others can be perfectly
    // content without their source files, such as how .js is valid
    // without a .coffee.
    
    fn get_source_files(&self) -> Vec<Path> {
        match self.ext {
            Some("class") => vec![self.path.with_extension("java")],  // Java
            Some("elc") => vec![self.path.with_extension("el")],  // Emacs Lisp
            Some("hi") => vec![self.path.with_extension("hs")],  // Haskell
            Some("o") => vec![self.path.with_extension("c"), self.path.with_extension("cpp")],  // C, C++
            Some("pyc") => vec![self.path.with_extension("py")],  // Python
            _ => vec![],
        }
    }
    
    fn get_source_files_usual(&self) -> Vec<Path> {
        match self.ext {
            Some("js") => vec![self.path.with_extension("coffee"), self.path.with_extension("ts")],  // CoffeeScript, TypeScript
            Some("css") => vec![self.path.with_extension("sass"), self.path.with_extension("less")],  // SASS, Less
            
            Some("aux") => vec![self.path.with_extension("tex")],  // TeX: auxiliary file
            Some("bbl") => vec![self.path.with_extension("tex")],  // BibTeX bibliography file
            Some("blg") => vec![self.path.with_extension("tex")],  // BibTeX log file
            Some("lof") => vec![self.path.with_extension("tex")],  // list of figures
            Some("log") => vec![self.path.with_extension("tex")],  // TeX log file
            Some("lot") => vec![self.path.with_extension("tex")],  // list of tables
            Some("toc") => vec![self.path.with_extension("tex")],  // table of contents

            _ => vec![],
        }
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
            Blue.bold()
        }
        else if self.stat.perm.contains(io::UserExecute) {
            Green.bold()
        }
        else if self.is_tmpfile() {
            Fixed(244).normal()  // midway between white and black - should show up as grey on all terminals
        }
        else if self.name.starts_with("README") {
            Yellow.bold().underline()
        }
        else if self.ext.is_some() && IMAGE_TYPES.iter().any(|&s| s == self.ext.unwrap()) {
            Fixed(133).normal()
        }
        else if self.ext.is_some() && VIDEO_TYPES.iter().any(|&s| s == self.ext.unwrap()) {
            Fixed(135).normal()
        }
        else if self.ext.is_some() && MUSIC_TYPES.iter().any(|&s| s == self.ext.unwrap()) {
            Fixed(92).normal()
        }
        else if self.ext.is_some() && MUSIC_LOSSLESS.iter().any(|&s| s == self.ext.unwrap()) {
            Fixed(93).normal()
        }
        else if self.ext.is_some() && CRYPTO_TYPES.iter().any(|&s| s == self.ext.unwrap()) {
            Fixed(109).normal()
        }
        else if self.ext.is_some() && DOCUMENT_TYPES.iter().any(|&s| s == self.ext.unwrap()) {
            Fixed(105).normal()
        }
        else if self.ext.is_some() && COMPRESSED_TYPES.iter().any(|&s| s == self.ext.unwrap()) {
            Red.normal()
        }
        else if self.ext.is_some() && TEMP_TYPES.iter().any(|&s| s == self.ext.unwrap()) {
            Fixed(244).normal()
        }
        else {
            let source_files = self.get_source_files();
            if source_files.len() == 0 {
                let source_files_usual = self.get_source_files_usual();
                if source_files_usual.iter().any(|path| self.dir.contains(path)) {
                    Fixed(244).normal()
                }
                else {
                    Plain
                }
            }
            else if source_files.iter().any(|path| self.dir.contains(path)) {
                Fixed(244).normal()
            }
            else {
                Fixed(137).normal()
            }
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
