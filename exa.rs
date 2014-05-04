extern crate getopts;
use std::io::fs;
use std::io;
use std::os;

use colours::{Plain, Style, Black, Red, Green, Yellow, Blue, Purple, Cyan};
mod colours;

struct Options {
    showInvisibles: bool,
}

fn main() {
    let args = os::args();
    let program = args[0].as_slice();
    let opts = ~[
        getopts::optflag("a", "all", "show dot-files")
    ];

    let matches = match getopts::getopts(args.tail(), opts) {
        Ok(m) => m,
        Err(f) => {
            fail!("Invalid options\n{}", f.to_err_msg());
            return
        }
    };

    let opts = Options {
        showInvisibles: matches.opt_present("all")
    };

    let strs = if matches.free.is_empty() {vec!(~"./")} else {matches.free.clone()};

    for dir in strs.move_iter() {
        list(opts, Path::new(dir))
    }
}

enum Permissions {
    Permissions,
}

enum FileName {
    FileName,
}

struct FileSize {
    useSIPrefixes: bool,
}

trait Column {
    fn display(&self, stat: &io::FileStat, filename: &str) -> ~str;
}

impl Column for FileName {
    fn display(&self, stat: &io::FileStat, filename: &str) -> ~str {
        file_colour(stat, filename).paint(filename.to_owned())
    }
}

impl Column for Permissions {
    fn display(&self, stat: &io::FileStat, filename: &str) -> ~str {
        let bits = stat.perm;
        return format!("{}{}{}{}{}{}{}{}{}{}",
            type_char(stat.kind),
            bit(bits, io::UserRead, ~"r", Yellow.bold()),
            bit(bits, io::UserWrite, ~"w", Red.bold()),
            bit(bits, io::UserExecute, ~"x", Green.bold().underline()),
            bit(bits, io::GroupRead, ~"r", Yellow.normal()),
            bit(bits, io::GroupWrite, ~"w", Red.normal()),
            bit(bits, io::GroupExecute, ~"x", Green.normal()),
            bit(bits, io::OtherRead, ~"r", Yellow.normal()),
            bit(bits, io::OtherWrite, ~"w", Red.normal()),
            bit(bits, io::OtherExecute, ~"x", Green.normal()),
       );
    }
}

impl Column for FileSize {
    fn display(&self, stat: &io::FileStat, filename: &str) -> ~str {
        let sizeStr = if self.useSIPrefixes {
            formatBytes(stat.size, 1024, ~[ "B  ", "KiB", "MiB", "GiB", "TiB" ])
        } else {
            formatBytes(stat.size, 1000, ~[ "B ", "KB", "MB", "GB", "TB" ])
        };

        return if stat.kind == io::TypeDirectory {
            Green.normal()
        } else {
            Green.bold()
        }.paint(sizeStr);
    }
}

fn formatBytes(mut amount: u64, kilo: u64, prefixes: ~[&str]) -> ~str {
    let mut prefix = 0;
    while amount > kilo {
        amount /= kilo;
        prefix += 1;
    }
    return format!("{:4}{}", amount, prefixes[prefix]);
}

// Each file is definitely going to get `stat`ted at least once, if
// only to determine what kind of file it is, so carry the `stat`
// result around with the file for safe keeping.
struct File<'a> {
    name: &'a str,
    path: &'a Path,
    stat: io::FileStat,
}

impl<'a> File<'a> {
    fn from_path(path: &'a Path) -> File<'a> {
        let filename: &str = path.filename_str().unwrap();

        // We have to use lstat here instad of file.stat(), as it
        // doesn't follow symbolic links. Otherwise, the stat() call
        // will fail if it encounters a link that's target is
        // non-existent.
        let stat: io::FileStat = match fs::lstat(path) {
            Ok(stat) => stat,
            Err(e) => fail!("Couldn't stat {}: {}", filename, e),
        };

        return File { path: path, stat: stat, name: filename };
    }
}

fn list(opts: Options, path: Path) {
    let mut files = match fs::readdir(&path) {
        Ok(files) => files,
        Err(e) => fail!("readdir: {}", e),
    };
    files.sort_by(|a, b| a.filename_str().cmp(&b.filename_str()));
    for subpath in files.iter() {
        let file = File::from_path(subpath);

        if file.name.starts_with(".") && !opts.showInvisibles {
            continue;
        }

        let columns = ~[
            ~Permissions as ~Column,
            ~FileSize { useSIPrefixes: false } as ~Column,
            ~FileName as ~Column
        ];

        let mut cells = columns.iter().map(|c| c.display(&file.stat, file.name));

        let mut first = true;
        for cell in cells {
            if first {
                first = false;
            } else {
                print!(" ");
            }
            print!("{}", cell);
        }
        print!("\n");
    }
}

fn file_colour(stat: &io::FileStat, filename: &str) -> Style {
    if stat.kind == io::TypeDirectory {
        Blue.normal()
    } else if stat.perm & io::UserExecute == io::UserExecute {
        Green.normal()
    } else if filename.ends_with("~") {
        Black.bold()
    } else {
        Plain
    }
}

fn bit(bits: u32, bit: u32, other: ~str, style: Style) -> ~str {
    if bits & bit == bit {
        style.paint(other)
    } else {
        Black.bold().paint(~"-")
    }
}

fn type_char(t: io::FileType) -> ~str {
    return match t {
        io::TypeFile => ~".",
        io::TypeDirectory => Blue.paint("d"),
        io::TypeNamedPipe => Yellow.paint("|"),
        io::TypeBlockSpecial => Purple.paint("s"),
        io::TypeSymlink => Cyan.paint("l"),
        _ => ~"?",
    }
}
