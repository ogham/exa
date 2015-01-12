use std::io::{fs, IoResult};
use std::io;

use ansi_term::{ANSIString, Colour, Style};
use ansi_term::Style::Plain;
use ansi_term::Colour::{Red, Green, Yellow, Blue, Purple, Cyan, Fixed};

use users::Users;

use number_prefix::{binary_prefix, decimal_prefix, Prefixed, Standalone, PrefixNames};

use column::{Column, SizeFormat};
use column::Column::*;
use dir::Dir;
use filetype::HasType;

pub static GREY: Colour = Fixed(244);

// Instead of working with Rust's Paths, we have our own File object
// that holds the Path and various cached information. Each file is
// definitely going to have its filename used at least once, its stat
// information queried at least once, and its file extension extracted
// at least once, so we may as well carry around that information with
// the actual path.

pub struct File<'a> {
    pub name:  String,
    pub dir:   Option<&'a Dir>,
    pub ext:   Option<String>,
    pub path:  Path,
    pub stat:  io::FileStat,
}

impl<'a> File<'a> {
    pub fn from_path(path: &Path, parent: Option<&'a Dir>) -> IoResult<File<'a>> {
        // Use lstat here instead of file.stat(), as it doesn't follow
        // symbolic links. Otherwise, the stat() call will fail if it
        // encounters a link that's target is non-existent.
        fs::lstat(path).map(|stat| File::with_stat(stat, path, parent))
    }

    pub fn with_stat(stat: io::FileStat, path: &Path, parent: Option<&'a Dir>) -> File<'a> {
        let v = path.filename().unwrap();  // fails if / or . or ..
        let filename = String::from_utf8_lossy(v);

        File {
            path:  path.clone(),
            dir:   parent,
            stat:  stat,
            name:  filename.to_string(),
            ext:   File::ext(filename.as_slice()),
        }
    }

    fn ext(name: &'a str) -> Option<String> {
        // The extension is the series of characters after a dot at
        // the end of a filename. This deliberately also counts
        // dotfiles - the ".git" folder has the extension "git".
        name.rfind('.').map(|pos| name.slice_from(pos + 1).to_string())
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
        if let Some(ref ext) = self.ext {
            let ext = ext.as_slice();
            match ext {
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

    pub fn display<U: Users>(&self, column: &Column, users_cache: &mut U) -> String {
        match *column {
            Permissions => {
                self.permissions_string()
            },

            FileName => {
                self.file_name()
            },

            FileSize(use_iec) => {
                self.file_size(use_iec)
            },

            // A file with multiple links is interesting, but
            // directories and suchlike can have multiple links all
            // the time.
            HardLinks => {
                let style = if self.has_multiple_links() { Red.on(Yellow) } else { Red.normal() };
                style.paint(self.stat.unstable.nlink.to_string().as_slice()).to_string()
            },

            Inode => {
                Purple.paint(self.stat.unstable.inode.to_string().as_slice()).to_string()
            },

            Blocks => {
                if self.stat.kind == io::FileType::RegularFile || self.stat.kind == io::FileType::Symlink {
                    Cyan.paint(self.stat.unstable.blocks.to_string().as_slice()).to_string()
                }
                else {
                    GREY.paint("-").to_string()
                }
            },

            // Display the ID if the user/group doesn't exist, which
            // usually means it was deleted but its files weren't.
            User => {
                let uid = self.stat.unstable.uid as i32;

                let user_name = match users_cache.get_user_by_uid(uid) {
                    Some(user) => user.name,
                    None => uid.to_string(),
                };

                let style = if users_cache.get_current_uid() == uid { Yellow.bold() } else { Plain };
                style.paint(user_name.as_slice()).to_string()
            },

            Group => {
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

                style.paint(group_name.as_slice()).to_string()
            },
        }
    }

    pub fn file_name(&self) -> String {
        let name = self.name.as_slice();
        let displayed_name = self.file_colour().paint(name);
        if self.stat.kind == io::FileType::Symlink {
            match fs::readlink(&self.path) {
                Ok(path) => {
                    let target_path = match self.dir {
                        Some(dir) => dir.path.join(path),
                        None => path,
                    };
                    format!("{} {}", displayed_name, self.target_file_name_and_arrow(&target_path))
                }
                Err(_) => displayed_name.to_string(),
            }
        }
        else {
            displayed_name.to_string()
        }
    }

    pub fn file_name_width(&self) -> usize {
        self.name.as_slice().width(false)
    }

    fn target_file_name_and_arrow(&self, target_path: &Path) -> String {
        let v = target_path.filename().unwrap();
        let filename = String::from_utf8_lossy(v);

        // Use stat instead of lstat - we *want* to follow links.
        let link_target = fs::stat(target_path).map(|stat| File {
            path:  target_path.clone(),
            dir:   self.dir,
            stat:  stat,
            name:  filename.to_string(),
            ext:   File::ext(filename.as_slice()),
        });

        // Statting a path usually fails because the file at the
        // other end doesn't exist. Show this by highlighting the
        // target file in red instead of displaying an error, because
        // the error would be shown out of context (before the
        // results, not right by the file) and it's almost always for
        // that reason anyway.

        match link_target {
            Ok(file) => format!("{} {}{}{}", GREY.paint("=>"), Cyan.paint(target_path.dirname_str().unwrap()), Cyan.paint("/"), file.file_colour().paint(filename.as_slice())),
            Err(_)   => format!("{} {}",     Red.paint("=>"),  Red.underline().paint(filename.as_slice())),
        }
    }

    fn file_size(&self, size_format: SizeFormat) -> String {
        // Don't report file sizes for directories. I've never looked
        // at one of those numbers and gained any information from it.
        if self.stat.kind == io::FileType::Directory {
            GREY.paint("-").to_string()
        }
        else {
            let result = match size_format {
                SizeFormat::DecimalBytes => decimal_prefix(self.stat.size as f64),
                SizeFormat::BinaryBytes  => binary_prefix(self.stat.size as f64),
                SizeFormat::JustBytes    => return Green.bold().paint(self.stat.size.to_string().as_slice()).to_string(),
            };

            match result {
                Standalone(bytes) => Green.bold().paint(bytes.to_string().as_slice()).to_string(),
                Prefixed(prefix, n) => {
                    let number = if n < 10f64 { format!("{:.1}", n) } else { format!("{:.0}", n) };
                    format!("{}{}", Green.bold().paint(number.as_slice()), Green.paint(prefix.symbol()))
                }
            }
        }
    }

    fn type_char(&self) -> ANSIString {
        return match self.stat.kind {
            io::FileType::RegularFile  => Plain.paint("."),
            io::FileType::Directory    => Blue.paint("d"),
            io::FileType::NamedPipe    => Yellow.paint("|"),
            io::FileType::BlockSpecial => Purple.paint("s"),
            io::FileType::Symlink      => Cyan.paint("l"),
            io::FileType::Unknown      => Plain.paint("?"),
        }
    }

    pub fn file_colour(&self) -> Style {
        self.get_type().style()
    }

    fn has_multiple_links(&self) -> bool {
        self.stat.kind == io::FileType::RegularFile && self.stat.unstable.nlink > 1
    }

    fn permissions_string(&self) -> String {
        let bits = self.stat.perm;
        let executable_colour = match self.stat.kind {
            io::FileType::RegularFile => Green.bold().underline(),
            _ => Green.bold(),
        };

        return format!("{}{}{}{}{}{}{}{}{}{}",
            self.type_char(),

            // The first three are bold because they're the ones used
            // most often.
            File::permission_bit(bits, io::USER_READ,     "r", Yellow.bold()),
            File::permission_bit(bits, io::USER_WRITE,    "w", Red.bold()),
            File::permission_bit(bits, io::USER_EXECUTE,  "x", executable_colour),
            File::permission_bit(bits, io::GROUP_READ,    "r", Yellow.normal()),
            File::permission_bit(bits, io::GROUP_WRITE,   "w", Red.normal()),
            File::permission_bit(bits, io::GROUP_EXECUTE, "x", Green.normal()),
            File::permission_bit(bits, io::OTHER_READ,    "r", Yellow.normal()),
            File::permission_bit(bits, io::OTHER_WRITE,   "w", Red.normal()),
            File::permission_bit(bits, io::OTHER_EXECUTE, "x", Green.normal()),
       );
    }

    fn permission_bit(bits: io::FilePermission, bit: io::FilePermission, character: &'static str, style: Style) -> ANSIString {
        if bits.contains(bit) {
            style.paint(character)
        }
        else {
            GREY.paint("-")
        }
    }
}
