use colours::{Plain, Style, Black, Red, Green, Yellow, Blue, Purple, Cyan};
use std::io::{fs, IoResult};
use std::io;

use column::{Column, Permissions, FileName, FileSize, User, Group};
use format::{format_metric_bytes, format_IEC_bytes};
use unix::{get_user_name, get_group_name};
use sort::SortPart;
use dir::Dir;
use filetype::HasType;

// Instead of working with Rust's Paths, we have our own File object
// that holds the Path and various cached information. Each file is
// definitely going to have its filename used at least once, its stat
// information queried at least once, and its file extension extracted
// at least once, so we may as well carry around that information with
// the actual path.

pub struct File<'a> {
    pub name:  &'a str,
    pub dir:   &'a Dir<'a>,
    pub ext:   Option<&'a str>,
    pub path:  &'a Path,
    pub stat:  io::FileStat,
    pub parts: Vec<SortPart>,
}

impl<'a> File<'a> {
    pub fn from_path(path: &'a Path, parent: &'a Dir) -> IoResult<File<'a>> {
        // Getting the string from a filename fails whenever it's not
        // UTF-8 representable - just assume it is for now.
        let filename: &str = path.filename_str().unwrap();

        // Use lstat here instead of file.stat(), as it doesn't follow
        // symbolic links. Otherwise, the stat() call will fail if it
        // encounters a link that's target is non-existent.
        
        fs::lstat(path).map(|stat| File {
            path:  path,
            dir:   parent,
            stat:  stat,
            name:  filename,
            ext:   File::ext(filename),
            parts: SortPart::split_into_parts(filename),
        })
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

    pub fn is_tmpfile(&self) -> bool {
        self.name.ends_with("~") || (self.name.starts_with("#") && self.name.ends_with("#"))
    }
        
    // Highlight the compiled versions of files. Some of them, like .o,
    // get special highlighting when they're alone because there's no
    // point in existing without their source. Others can be perfectly
    // content without their source files, such as how .js is valid
    // without a .coffee.
    
    pub fn get_source_files(&self) -> Vec<Path> {
        match self.ext {
            Some("class") => vec![self.path.with_extension("java")],  // Java
            Some("elc") => vec![self.path.with_extension("el")],  // Emacs Lisp
            Some("hi") => vec![self.path.with_extension("hs")],  // Haskell
            Some("o") => vec![self.path.with_extension("c"), self.path.with_extension("cpp")],  // C, C++
            Some("pyc") => vec![self.path.with_extension("py")],  // Python
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
            FileName => self.file_name(),
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
    
    fn file_name(&self) -> String {
        let displayed_name = self.file_colour().paint(self.name);
        if self.stat.kind == io::TypeSymlink {
            match fs::readlink(self.path) {
                Ok(path) => format!("{} => {}", displayed_name, path.display()),
                Err(e) => {
                    println!("{}", e);
                    displayed_name
                },
            }
        }
        else {
            displayed_name
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
        self.get_type().style()
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
