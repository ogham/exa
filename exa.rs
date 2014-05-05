extern crate getopts;
use std::os;
use std::io::fs;

use file::File;
use column::defaultColumns;

pub mod colours;
pub mod column;
pub mod format;
pub mod file;

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
        Err(f) => fail!("Invalid options\n{}", f.to_err_msg()),
    };

    let opts = Options {
        showInvisibles: matches.opt_present("all")
    };

    let strs = if matches.free.is_empty() {
        vec!("./".to_owned())
    }
    else {
        matches.free.clone()
    };

    for dir in strs.move_iter() {
        list(opts, Path::new(dir))
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

        if file.is_dotfile() && !opts.showInvisibles {
            continue;
        }

        let columns = defaultColumns();

        let mut cells = columns.iter().map(|c| file.display(c));

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
