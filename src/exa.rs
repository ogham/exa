#![allow(unstable)]

extern crate ansi_term;
extern crate number_prefix;
extern crate unicode;
extern crate users;

use std::io::FileType;
use std::io::fs;
use std::os::{args, set_exit_status};

use dir::Dir;
use file::File;
use options::Options;
use options::Error::*;

pub mod column;
pub mod dir;
pub mod file;
pub mod filetype;
pub mod options;
pub mod output;
pub mod term;

fn exa(options: &Options) {
    let mut dirs: Vec<String> = vec![];
    let mut files: Vec<File> = vec![];

    // It's only worth printing out directory names if the user supplied
    // more than one of them.
    let mut print_dir_names = false;

    // Separate the user-supplied paths into directories and files.
    // Files are shown first, and then each directory is expanded
    // and listed second.
    for file in options.path_strings() {
        let path = Path::new(file);
        match fs::stat(&path) {
            Ok(stat) => {
                if !options.list_dirs && stat.kind == FileType::Directory {
                    dirs.push(file.clone());
                }
                else {
                    // May as well reuse the stat result from earlier
                    // instead of just using File::from_path().
                    files.push(File::with_stat(stat, &path, None));
                }
            }
            Err(e) => println!("{}: {}", file, e),
        }

        print_dir_names = true;
    }

    let mut first = files.is_empty();

    if !files.is_empty() {
        options.view.view(files);
    }

    for dir_name in dirs.iter() {
        if first {
            first = false;
        }
        else {
            print!("\n");
        }

        match Dir::readdir(Path::new(dir_name.clone())) {
            Ok(dir) => {
                let unsorted_files = dir.files();
                let files: Vec<File> = options.transform_files(unsorted_files);

                if print_dir_names {
                    println!("{}:", dir_name);
                }

                options.view.view(files);
            }
            Err(e) => {
                println!("{}: {}", dir_name, e);
                return;
            }
        };
    }
}

fn main() {
    let args: Vec<String> = args();

    match Options::getopts(args) {
        Ok(options) => exa(&options),
        Err(Help(text)) => {
            println!("{}", text);
            set_exit_status(2);
        },
        Err(InvalidOptions(e)) => {
            println!("{}", e);
            set_exit_status(3);
        },
    };
}
