#![feature(convert, fs_mode)]
#![feature(slice_extras, iter_arith, vec_resize)]

extern crate ansi_term;
extern crate datetime;
extern crate getopts;
extern crate libc;
extern crate locale;
extern crate natord;
extern crate num_cpus;
extern crate number_prefix;
extern crate pad;
extern crate threadpool;
extern crate unicode_width;
extern crate users;

#[cfg(feature="git")]
extern crate git2;


use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process;
use std::sync::mpsc::channel;

use threadpool::ThreadPool;

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


#[cfg(not(test))]
struct Exa<'dir> {
    count:   usize,
    options: Options,
    dirs:    Vec<PathBuf>,
    files:   Vec<File<'dir>>,
}

#[cfg(not(test))]
impl<'dir> Exa<'dir> {
    fn new(options: Options) -> Exa<'dir> {
        Exa {
            count: 0,
            options: options,
            dirs: Vec::new(),
            files: Vec::new(),
        }
    }

    fn load(&mut self, files: &[String]) {

        // Separate the user-supplied paths into directories and files.
        // Files are shown first, and then each directory is expanded
        // and listed second.
        let is_tree = self.options.dir_action.is_tree() || self.options.dir_action.is_as_file();
        let total_files = files.len();


        // Communication between consumer thread and producer threads
        enum StatResult<'dir> {
            File(File<'dir>),
            Dir(PathBuf),
            Error
        }

        let pool = ThreadPool::new(8 * num_cpus::get());
        let (tx, rx) = channel();

        for file in files.iter() {
            let tx = tx.clone();
            let file = file.clone();

            // Spawn producer thread
            pool.execute(move || {
                let path = Path::new(&*file);
                let _ = tx.send(match fs::metadata(&path) {
                    Ok(metadata) => {
                        if !metadata.is_dir() {
                            StatResult::File(File::with_metadata(metadata, &path, None, false))
                        }
                        else if is_tree {
                            StatResult::File(File::with_metadata(metadata, &path, None, true))
                        }
                        else {
                            StatResult::Dir(path.to_path_buf())
                        }
                    }
                    Err(e) => {
                        println!("{}: {}", file, e);
                        StatResult::Error
                    }
                });
            });
        }

        // Spawn consumer thread
        for result in rx.iter().take(total_files) {
            match result {
                StatResult::File(file)  => self.files.push(file),
                StatResult::Dir(path)   => self.dirs.push(path),
                StatResult::Error       => ()
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
                    if let Some(recurse_opts) = self.options.dir_action.recurse_options() {
                        let depth = dir_path.components().filter(|&c| c != Component::CurDir).count() + 1;
                        if !recurse_opts.tree && !recurse_opts.is_too_deep(depth) {
                            for dir in files.iter().filter(|f| f.is_directory()).rev() {
                                self.dirs.push(dir.path.clone());
                            }
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
            View::Lines(l)    => l.view(files),
        }
    }
}


#[cfg(not(test))]
fn main() {
    let args: Vec<String> = env::args().collect();

    match Options::getopts(args.tail()) {
        Ok((options, paths)) => {
            let mut exa = Exa::new(options);
            exa.load(&paths);
            exa.print_files();
            exa.print_dirs();
        },
        Err(e) => {
            println!("{}", e);
            process::exit(e.error_code());
        },
    };
}
