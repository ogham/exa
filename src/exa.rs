#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_results)]

extern crate ansi_term;
extern crate datetime;
extern crate getopts;
extern crate glob;
extern crate libc;
extern crate locale;
extern crate natord;
extern crate num_cpus;
extern crate number_prefix;
extern crate scoped_threadpool;
extern crate term_grid;
extern crate unicode_width;
extern crate users;
extern crate zoneinfo_compiled;

#[cfg(feature="git")] extern crate git2;

#[macro_use]
extern crate lazy_static;


use std::ffi::OsStr;
use std::io::{stderr, Write, Result as IOResult};
use std::path::{Component, Path};

use ansi_term::{ANSIStrings, Style};

use fs::{Dir, File};
use options::{Options, View, Mode};
pub use options::Misfire;
use output::{escape, lines, grid, grid_details, details};

mod fs;
mod info;
mod options;
mod output;
mod term;


/// The main program wrapper.
pub struct Exa<'w, W: Write + 'w> {

    /// List of command-line options, having been successfully parsed.
    pub options: Options,

    /// The output handle that we write to. When running the program normally,
    /// this will be `std::io::Stdout`, but it can accept any struct that’s
    /// `Write` so we can write into, say, a vector for testing.
    pub writer: &'w mut W,

    /// List of the free command-line arguments that should correspond to file
    /// names (anything that isn’t an option).
    pub args: Vec<String>,
}

impl<'w, W: Write + 'w> Exa<'w, W> {
    pub fn new<C>(args: C, writer: &'w mut W) -> Result<Exa<'w, W>, Misfire>
    where C: IntoIterator, C::Item: AsRef<OsStr> {
        Options::getopts(args).map(move |(options, args)| {
            Exa { options, writer, args }
        })
    }

    pub fn run(&mut self) -> IOResult<i32> {
        let mut files = Vec::new();
        let mut dirs = Vec::new();
        let mut exit_status = 0;

        // List the current directory by default, like ls.
        if self.args.is_empty() {
            self.args.push(".".to_owned());
        }

        for file_name in &self.args {
            match File::from_path(Path::new(&file_name), None) {
                Err(e) => {
                    exit_status = 2;
                    writeln!(stderr(), "{}: {}", file_name, e)?;
                },
                Ok(f) => {
                    if f.is_directory() && !self.options.dir_action.treat_dirs_as_files() {
                        match f.to_dir(self.options.should_scan_for_git()) {
                            Ok(d) => dirs.push(d),
                            Err(e) => writeln!(stderr(), "{}: {}", file_name, e)?,
                        }
                    }
                    else {
                        files.push(f);
                    }
                },
            }
        }

        // We want to print a directory’s name before we list it, *except* in
        // the case where it’s the only directory, *except* if there are any
        // files to print as well. (It’s a double negative)

        let no_files = files.is_empty();
        let is_only_dir = dirs.len() == 1 && no_files;

        self.options.filter.filter_argument_files(&mut files);
        self.print_files(None, files)?;

        self.print_dirs(dirs, no_files, is_only_dir, exit_status)
    }

    fn print_dirs(&mut self, dir_files: Vec<Dir>, mut first: bool, is_only_dir: bool, exit_status: i32) -> IOResult<i32> {
        for dir in dir_files {

            // Put a gap between directories, or between the list of files and
            // the first directory.
            if first {
                first = false;
            }
            else {
                write!(self.writer, "\n")?;
            }

            if !is_only_dir {
                let mut bits = Vec::new();
                escape(dir.path.display().to_string(), &mut bits, Style::default(), Style::default());
                writeln!(self.writer, "{}:", ANSIStrings(&bits))?;
            }

            let mut children = Vec::new();
            for file in dir.files() {
                match file {
                    Ok(file)       => children.push(file),
                    Err((path, e)) => writeln!(stderr(), "[{}: {}]", path.display(), e)?,
                }
            };

            self.options.filter.filter_child_files(&mut children);
            self.options.filter.sort_files(&mut children);

            if let Some(recurse_opts) = self.options.dir_action.recurse_options() {
                let depth = dir.path.components().filter(|&c| c != Component::CurDir).count() + 1;
                if !recurse_opts.tree && !recurse_opts.is_too_deep(depth) {

                    let mut child_dirs = Vec::new();
                    for child_dir in children.iter().filter(|f| f.is_directory()) {
                        match child_dir.to_dir(false) {
                            Ok(d)  => child_dirs.push(d),
                            Err(e) => writeln!(stderr(), "{}: {}", child_dir.path.display(), e)?,
                        }
                    }

                    self.print_files(Some(&dir), children)?;
                    match self.print_dirs(child_dirs, false, false, exit_status) {
                        Ok(_) => (),
                        Err(e) => return Err(e),
                    }
                    continue;
                }
            }

            self.print_files(Some(&dir), children)?;
        }

        Ok(exit_status)
    }

    /// Prints the list of files using whichever view is selected.
    /// For various annoying logistical reasons, each one handles
    /// printing differently...
    fn print_files(&mut self, dir: Option<&Dir>, files: Vec<File>) -> IOResult<()> {
        if !files.is_empty() {
            let View { ref mode, ref colours, classify } = self.options.view;

            match *mode {
                Mode::Lines                  => lines::Render { files, colours, classify }.render(self.writer),
                Mode::Grid(ref opts)         => grid::Render { files, colours, classify, opts }.render(self.writer),
                Mode::Details(ref opts)      => details::Render { dir, files, colours, classify, opts }.render(self.writer),
                Mode::GridDetails(ref grid, ref details) => grid_details::Render { dir, files, colours, classify, grid, details }.render(self.writer),
            }
        }
        else {
            Ok(())
        }
    }
}
