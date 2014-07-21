use std::io::{fs, IoResult};
use std::io;

use ansi_term::{Paint, Colour, Plain, Style, Red, Green, Yellow, Blue, Purple, Cyan, Fixed};

use column::{Column, Permissions, FileName, FileSize, User, Group, HardLinks, Inode, Blocks};
use format::{format_metric_bytes, format_IEC_bytes};
use unix::Unix;
use sort::SortPart;
use dir::Dir;
use filetype::HasType;

static Grey: Colour = Fixed(244);

// Instead of working with Rust's Paths, we have our own File object
// that holds the Path and various cached information. Each file is
// definitely going to have its filename used at least once, its stat
// information queried at least once, and its file extension extracted
// at least once, so we may as well carry around that information with
// the actual path.

pub struct File<'a> {
    pub name:  String,
    pub dir:   &'a Dir<'a>,
    pub ext:   Option<String>,
    pub path:  &'a Path,
    pub stat:  io::FileStat,
    pub parts: Vec<SortPart>,
}

impl<'a> File<'a> {
    pub fn from_path(path: &'a Path, parent: &'a Dir) -> IoResult<File<'a>> {
        let v = path.filename().unwrap();  // fails if / or . or ..
        let filename = String::from_utf8_lossy(v).to_string();
        
        // Use lstat here instead of file.stat(), as it doesn't follow
        // symbolic links. Otherwise, the stat() call will fail if it
        // encounters a link that's target is non-existent.

        fs::lstat(path).map(|stat| File {
            path:  path,
            dir:   parent,
            stat:  stat,
            name:  filename.clone(),
            ext:   File::ext(filename.clone()),
            parts: SortPart::split_into_parts(filename.clone()),
        })
    }

    fn ext(name: String) -> Option<String> {
        // The extension is the series of characters after a dot at
        // the end of a filename. This deliberately also counts
        // dotfiles - the ".git" folder has the extension "git".
        let re = regex!(r"\.([^.]+)$");
        re.captures(name.as_slice()).map(|caps| caps.at(1).to_string())
    }

    pub fn is_dotfile(&self) -> bool {
        self.name.as_slice().starts_with(".")
    }

    pub fn is_tmpfile(&self) -> bool {
        let name = self.name.as_slice();
        name.ends_with("~") || (name.starts_with("#") && name.ends_with("#"))
    }

    // Highlight the compiled versions of files. Some of them, like .o,
    // get special highlighting when they're alone because there's no
    // point in existing without their source. Others can be perfectly
    // content without their source files, such as how .js is valid
    // without a .coffee.

    pub fn get_source_files(&self) -> Vec<Path> {
        if self.ext.is_none() {
            return vec![];
        }
        
        let ext = self.ext.clone().unwrap();
        match ext.as_slice() {
            "class" => vec![self.path.with_extension("java")],  // Java
            "elc" => vec![self.path.with_extension("el")],  // Emacs Lisp
            "hi" => vec![self.path.with_extension("hs")],  // Haskell
            "o" => vec![self.path.with_extension("c"), self.path.with_extension("cpp")],  // C, C++
            "pyc" => vec![self.path.with_extension("py")],  // Python
            "js" => vec![self.path.with_extension("coffee"), self.path.with_extension("ts")],  // CoffeeScript, TypeScript
            "css" => vec![self.path.with_extension("sass"), self.path.with_extension("less")],  // SASS, Less

            "aux" => vec![self.path.with_extension("tex")],  // TeX: auxiliary file
            "bbl" => vec![self.path.with_extension("tex")],  // BibTeX bibliography file
            "blg" => vec![self.path.with_extension("tex")],  // BibTeX log file
            "lof" => vec![self.path.with_extension("tex")],  // list of figures
            "log" => vec![self.path.with_extension("tex")],  // TeX log file
            "lot" => vec![self.path.with_extension("tex")],  // list of tables
            "toc" => vec![self.path.with_extension("tex")],  // table of contents

            _ => vec![],
        }
    }

    pub fn display(&self, column: &Column, unix: &mut Unix) -> String {
        match *column {
            Permissions => self.permissions_string(),
            FileName => self.file_name(),
            FileSize(use_iec) => self.file_size(use_iec),

            // A file with multiple links is interesting, but
            // directories and suchlike can have multiple links all
            // the time.
            HardLinks => {
                let style = if self.stat.kind == io::TypeFile && self.stat.unstable.nlink > 1 { Red.on(Yellow) } else { Red.normal() };
                style.paint(self.stat.unstable.nlink.to_string().as_slice())
            },

            Inode => Purple.paint(self.stat.unstable.inode.to_string().as_slice()),
            Blocks => {
                if self.stat.kind == io::TypeFile || self.stat.kind == io::TypeSymlink {
                    Cyan.paint(self.stat.unstable.blocks.to_string().as_slice())
                }
                else {
                    Grey.paint("-")
                }
            },

            // Display the ID if the user/group doesn't exist, which
            // usually means it was deleted but its files weren't.
            User => {
                let uid = self.stat.unstable.uid as u32;
                unix.load_user(uid);
                let style = if unix.uid == uid { Yellow.bold() } else { Plain };
                let string = unix.get_user_name(uid).unwrap_or(uid.to_string());
                style.paint(string.as_slice())
            },
            Group => {
                let gid = self.stat.unstable.gid as u32;
                unix.load_group(gid);
                let name = unix.get_group_name(gid).unwrap_or(gid.to_string());
                let style = if unix.is_group_member(gid) { Yellow.normal() } else { Plain };
                style.paint(name.as_slice())
            },
        }
    }

    fn file_name(&self) -> String {
        let name = self.name.as_slice();
        let displayed_name = self.file_colour().paint(name);
        if self.stat.kind == io::TypeSymlink {
            match fs::readlink(self.path) {
                Ok(path) => {
                    let target_path = if path.is_absolute() { path } else { self.dir.path.join(path) };
                    format!("{} {}", displayed_name, self.target_file_name_and_arrow(target_path))
                }
                Err(_) => displayed_name,
            }
        }
        else {
            displayed_name
        }
    }

    fn target_file_name_and_arrow(&self, target_path: Path) -> String {
        let v = target_path.filename().unwrap();
        let filename = String::from_utf8_lossy(v).to_string();
        
        let link_target = fs::stat(&target_path).map(|stat| File {
            path:  &target_path,
            dir:   self.dir,
            stat:  stat,
            name:  filename.clone(),
            ext:   File::ext(filename.clone()),
            parts: vec![],  // not needed
        });

        // Statting a path usually fails because the file at the other
        // end doesn't exist. Show this by highlighting the target
        // file in red instead of displaying an error, because the
        // error would be shown out of context and it's almost always
        // that reason anyway.

        match link_target {
            Ok(file) => format!("{} {}", Grey.paint("=>"), file.file_colour().paint(filename.as_slice())),
            Err(_)   => format!("{} {}", Red.paint("=>"), Red.underline().paint(filename.as_slice())),
        }
    }

    fn file_size(&self, use_iec_prefixes: bool) -> String {
        // Don't report file sizes for directories. I've never looked
        // at one of those numbers and gained any information from it.
        if self.stat.kind == io::TypeDirectory {
            Grey.paint("-")
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
            io::TypeUnknown      => "?".to_string(),
        }
    }

    pub fn file_colour(&self) -> Style {
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
            Grey.paint("-".as_slice())
        }
    }
}
