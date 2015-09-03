#![feature(iter_arith)]
#![feature(convert, fs_mode)]
#![feature(slice_splits, vec_resize)]

extern crate ansi_term;
extern crate datetime;
extern crate getopts;
extern crate libc;
extern crate locale;
extern crate natord;
extern crate num_cpus;
extern crate number_prefix;
extern crate pad;
extern crate scoped_threadpool;
extern crate term_grid;
extern crate unicode_width;
extern crate users;

#[cfg(feature="git")]
extern crate git2;


use std::env;
use std::path::{Component, Path};
use std::process;

use dir::Dir;
use file::File;
use options::{Options, View};

mod colours;
mod column;
mod dir;
mod feature;
mod file;
mod filetype;
mod options;
mod output;
mod term;


struct Exa {
    options: Options,
}

impl Exa {
    fn run(&mut self, args_file_names: &[String]) {
        let mut files = Vec::new();
        let mut dirs = Vec::new();

        for file_name in args_file_names.iter() {
            match File::from_path(Path::new(&file_name), None) {
                Err(e) => {
                    println!("{}: {}", file_name, e);
                },
                Ok(f) => {
                    if f.is_directory() && !self.options.dir_action.treat_dirs_as_files() {
                        match f.to_dir(self.options.should_scan_for_git()) {
                            Ok(d) => dirs.push(d),
                            Err(e) => println!("{}: {}", file_name, e),
                        }
                    }
                    else {
                        files.push(f);
                    }
                },
            }
        }

        let any_files = files.is_empty();
        self.print_files(None, files);

        let is_only_dir = dirs.len() == 1;
        self.print_dirs(dirs, any_files, is_only_dir);
    }

    fn print_dirs(&self, dir_files: Vec<Dir>, mut first: bool, is_only_dir: bool) {
        for dir in dir_files {

            // Put a gap between directories, or between the list of files and the
            // first directory.
            if first {
                first = false;
            }
            else {
                print!("\n");
            }

            if !is_only_dir {
                println!("{}:", dir.path.display());
            }

            let mut children = Vec::new();
            for file in dir.files() {
                match file {
                    Ok(file)       => children.push(file),
                    Err((path, e)) => println!("[{}: {}]", path.display(), e),
                }
            };

            self.options.filter_files(&mut children);
            self.options.sort_files(&mut children);

            if let Some(recurse_opts) = self.options.dir_action.recurse_options() {
                let depth = dir.path.components().filter(|&c| c != Component::CurDir).count() + 1;
                if !recurse_opts.tree && !recurse_opts.is_too_deep(depth) {

                    let mut child_dirs = Vec::new();
                    for child_dir in children.iter().filter(|f| f.is_directory()) {
                        match child_dir.to_dir(false) {
                            Ok(d)  => child_dirs.push(d),
                            Err(e) => println!("{}: {}", child_dir.path.display(), e),
                        }
                    }

                    self.print_files(Some(&dir), children);

                    if !child_dirs.is_empty() {
                        self.print_dirs(child_dirs, false, false);
                    }

                    continue;
                }
            }

            self.print_files(Some(&dir), children);

        }
    }

    fn print_files(&self, dir: Option<&Dir>, files: Vec<File>) {
        match self.options.view {
            View::Grid(g)         => g.view(&files),
            View::Details(d)      => d.view(dir, files),
            View::GridDetails(gd) => gd.view(dir, &files),
            View::Lines(l)        => l.view(&files),
        }
    }
}


fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match Options::getopts(&args) {
        Ok((options, paths)) => {
            let mut exa = Exa { options: options };
            exa.run(&paths);
        },
        Err(e) => {
            println!("{}", e);
            process::exit(e.error_code());
        },
    };
}
