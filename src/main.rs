#![feature(collections, core, io, libc, os, path, std_misc)]

extern crate ansi_term;
extern crate getopts;
extern crate natord;
extern crate number_prefix;
extern crate users;

#[cfg(feature="git")]
extern crate git2;

use std::old_io::{fs, FileType};
use std::os::{args, set_exit_status};

use dir::Dir;
use file::File;
use options::{Options, DirAction};

pub mod column;
pub mod dir;
pub mod file;
pub mod filetype;
pub mod options;
pub mod output;
pub mod term;

fn exa(options: &Options) {
    let mut dirs: Vec<Path> = vec![];
    let mut files: Vec<File> = vec![];

    // It's only worth printing out directory names if the user supplied
    // more than one of them.
    let mut count = 0;

    // Separate the user-supplied paths into directories and files.
    // Files are shown first, and then each directory is expanded
    // and listed second.
    for file in options.path_strs.iter() {
        let path = Path::new(file);
        match fs::stat(&path) {
            Ok(stat) => {
                if stat.kind == FileType::Directory && options.dir_action != DirAction::AsFile {
                    dirs.push(path);
                }
                else {
                    files.push(File::with_stat(stat, &path, None));
                }
            }
            Err(e) => println!("{}: {}", file, e),
        }

        count += 1;
    }

    let mut first = files.is_empty();

    if !files.is_empty() {
        options.view(None, &files[]);
    }

    // Directories are put on a stack rather than just being iterated through,
    // as the vector can change as more directories are added.
    loop {
        let dir_path = match dirs.pop() {
            None => break,
            Some(f) => f,
        };

        // Put a gap between directories, or between the list of files and the
        // first directory.
        if first {
            first = false;
        }
        else {
            print!("\n");
        }

        match Dir::readdir(&dir_path) {
            Ok(ref dir) => {
                let unsorted_files = dir.files();
                let files: Vec<File> = options.transform_files(unsorted_files);

                // When recursing, add any directories to the dirs stack
                // backwards: the *last* element of the stack is used each
                // time, so by inserting them backwards, they get displayed in
                // the correct sort order.
                if options.dir_action == DirAction::Recurse {
                    for dir in files.iter().filter(|f| f.stat.kind == FileType::Directory).rev() {
                        dirs.push(dir.path.clone());
                    }
                }

                if count > 1 {
                    println!("{}:", dir_path.display());
                }
                count += 1;

                options.view(Some(dir), &files[]);
            }
            Err(e) => {
                println!("{}: {}", dir_path.display(), e);
                return;
            }
        };
    }
}

fn main() {
    let args: Vec<String> = args();

    match Options::getopts(args.tail()) {
        Ok(options) => exa(&options),
        Err(e) => {
            println!("{}", e);
            set_exit_status(e.error_code());
        },
    };
}
