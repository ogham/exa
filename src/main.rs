#![feature(collections, core, exit_status, io, libc, old_io, old_path, os, std_misc)]

// Other platforms than macos don't need std_misc but you can't
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
use std::os::num_cpus;
use std::sync::mpsc::{channel, sync_channel};
use std::thread;

use dir::Dir;
use file::File;
use options::{Options, View};
use output::lines_view;

pub mod column;
pub mod dir;
pub mod file;
pub mod filetype;
pub mod options;
pub mod output;
pub mod term;
pub mod xattr;

#[cfg(not(test))]
struct Exa<'a> {
    count:   usize,
    options: Options,
    dirs:    Vec<Path>,
    files:   Vec<File<'a>>,
}

#[cfg(not(test))]
impl<'a> Exa<'a> {
    fn new(options: Options) -> Exa<'a> {
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

        let is_tree = self.options.dir_action.is_tree();
        let total_files = files.len();

        // Denotes the maxinum number of concurrent threads
        let (thread_capacity_tx, thread_capacity_rs) = sync_channel(8 * num_cpus());

        // Communication between consumer thread and producer threads
        enum StatResult<'a> {
            File(File<'a>),
            Path(Path)
        }
        let (results_tx, results_rx) = channel();

        // Spawn consumer thread
        let _consumer = thread::scoped(move || {
            for _ in 0..total_files {

                // Make room for more producer threads
                let _ = thread_capacity_rs.recv();

                // Receive a producer's result
                match results_rx.recv() {
                    Ok(result) => match result {
                        StatResult::File(file) => self.files.push(file),
                        StatResult::Path(path) => self.dirs.push(path)
                    },
                    Err(_) => unreachable!()
                }
                self.count += 1;
            }
        });

        for file in files.iter() {
            let file = file.clone();
            let results_tx = results_tx.clone();

            // Block until there is room for another thread
            let _ = thread_capacity_tx.send(());

            // Spawn producer thread
            thread::spawn(move || {
                let path = Path::new(file.clone());
                match fs::stat(&path) {
                    Ok(stat) => {
                        if stat.kind == FileType::Directory {
                            if is_tree {
                                let _ = results_tx.send(StatResult::File(File::with_stat(stat, &path, None, true)));
                            }
                            else {
                                let _ = results_tx.send(StatResult::Path(path));
                            }
                        }
                        else {
                            let _ = results_tx.send(StatResult::File(File::with_stat(stat, &path, None, false)));
                        }
                    }
                    Err(e) => println!("{}: {}", file, e),
                }
            });
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
                        let depth = dir_path.components().filter(|&c| c != b".").count() + 1;
                        if !recurse_opts.tree && !recurse_opts.is_too_deep(depth) {
                            for dir in files.iter().filter(|f| f.stat.kind == FileType::Directory).rev() {
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
            View::Lines       => lines_view(files),
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
            env::set_exit_status(e.error_code());
        },
    };
}
