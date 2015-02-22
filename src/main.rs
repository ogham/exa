#![feature(collections, core, env, libc, old_io, old_path, plugin, std_misc)]
// Other platforms then macos don’t need std_misc but you can’t 
// use #[cfg] on features.
#![allow(unused_features)] 

extern crate ansi_term;
extern crate datetime;
extern crate getopts;
extern crate locale;
extern crate natord;
extern crate number_prefix;
extern crate pad;
extern crate users;

#[cfg(feature="git")]
extern crate git2;

use std::env;
use std::old_io::{fs, FileType};

use dir::Dir;
use file::File;
use options::{Options, View, DirAction};
use output::lines_view;

pub mod column;
pub mod dir;
pub mod file;
pub mod filetype;
pub mod options;
pub mod output;
pub mod term;
pub mod attr;

struct Exa<'a> {
    count:   usize,
    options: Options,
    dirs:    Vec<Path>,
    files:   Vec<File<'a>>,
}

impl<'a> Exa<'a> {
    fn new(options: Options) -> Exa<'a> {
        Exa {
            count: 0,
            options: options,
            dirs: Vec::new(),
            files: Vec::new(),
        }
    }

    fn load<T>(&mut self, iter: T) where T: Iterator<Item = &'a String> {
        // Separate the user-supplied paths into directories and files.
        // Files are shown first, and then each directory is expanded
        // and listed second.
        for file in iter {
            let path = Path::new(file);
            match fs::stat(&path) {
                Ok(stat) => {
                    if stat.kind == FileType::Directory {
                        if self.options.dir_action == DirAction::Tree {
                            self.files.push(File::with_stat(stat, &path, None, true));
                        }
                        else {
                            self.dirs.push(path);
                        }
                    }
                    else {
                        self.files.push(File::with_stat(stat, &path, None, false));
                    }
                }
                Err(e) => println!("{}: {}", file, e),
            }

            self.count += 1;
        }
    }

    fn print_files(&self) {
        if !self.files.is_empty() {
            self.print(None, &self.files[..]);
        }
    }

    fn print_dirs(&mut self) {
        let mut first = self.files.is_empty();

        // Directories are put on a stack rather than just being iterated through,
        // as the vector can change as more directories are added.
        loop {
            let dir_path = match self.dirs.pop() {
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
                    let mut files = dir.files(false);
                    self.options.transform_files(&mut files);

                    // When recursing, add any directories to the dirs stack
                    // backwards: the *last* element of the stack is used each
                    // time, so by inserting them backwards, they get displayed in
                    // the correct sort order.
                    if self.options.dir_action == DirAction::Recurse {
                        for dir in files.iter().filter(|f| f.stat.kind == FileType::Directory).rev() {
                            self.dirs.push(dir.path.clone());
                        }
                    }

                    if self.count > 1 {
                        println!("{}:", dir_path.display());
                    }
                    self.count += 1;

                    self.print(Some(dir), &files[..]);
                }
                Err(e) => {
                    println!("{}: {}", dir_path.display(), e);
                    return;
                }
            };
        }
    }

    fn print(&self, dir: Option<&Dir>, files: &[File]) {
        match self.options.view {
            View::Grid(g)     => g.view(files),
            View::Details(d)  => d.view(dir, files),
            View::Lines       => lines_view(files),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match Options::getopts(args.tail()) {
        Ok((options, paths)) => {
            let mut exa = Exa::new(options);
            exa.load(paths.iter());
            exa.print_files();
            exa.print_dirs();
        },
        Err(e) => {
            println!("{}", e);
            env::set_exit_status(e.error_code());
        },
    };
}
