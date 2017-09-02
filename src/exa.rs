#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_results)]

extern crate ansi_term;
extern crate datetime;
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
extern crate term_size;

#[cfg(feature="git")] extern crate git2;

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;


use std::env::var_os;
use std::ffi::{OsStr, OsString};
use std::io::{stderr, Write, Result as IOResult};
use std::path::{Component, PathBuf};

use ansi_term::{ANSIStrings, Style};

use fs::{Dir, File};
use fs::feature::git::GitCache;
use options::{Options, Vars};
pub use options::Misfire;
use output::{escape, lines, grid, grid_details, details, View, Mode};

mod fs;
mod info;
mod options;
mod output;
mod style;


/// The main program wrapper.
pub struct Exa<'args, 'w, W: Write + 'w> {

    /// List of command-line options, having been successfully parsed.
    pub options: Options,

    /// The output handle that we write to. When running the program normally,
    /// this will be `std::io::Stdout`, but it can accept any struct that’s
    /// `Write` so we can write into, say, a vector for testing.
    pub writer: &'w mut W,

    /// List of the free command-line arguments that should correspond to file
    /// names (anything that isn’t an option).
    pub args: Vec<&'args OsStr>,

    /// A global Git cache, if the option was passed in.
    /// This has to last the lifetime of the program, because the user might
    /// want to list several directories in the same repository.
    pub git: Option<GitCache>,
}

/// The “real” environment variables type.
/// Instead of just calling `var_os` from within the options module,
/// the method of looking up environment variables has to be passed in.
struct LiveVars;
impl Vars for LiveVars {
    fn get(&self, name: &'static str) -> Option<OsString> {
        var_os(name)
    }
}

/// Create a Git cache populated with the arguments that are going to be
/// listed before they’re actually listed, if the options demand it.
fn git_options(options: &Options, args: &[&OsStr]) -> Option<GitCache> {
    if options.should_scan_for_git() {
        Some(args.iter().map(|os| PathBuf::from(os)).collect())
    }
    else {
        None
    }
}

impl<'args, 'w, W: Write + 'w> Exa<'args, 'w, W> {
    pub fn new<I>(args: I, writer: &'w mut W) -> Result<Exa<'args, 'w, W>, Misfire>
    where I: Iterator<Item=&'args OsString> {
        Options::parse(args, &LiveVars).map(move |(options, mut args)| {
            debug!("Dir action from arguments: {:#?}", options.dir_action);
            debug!("Filter from arguments: {:#?}", options.filter);
            debug!("View from arguments: {:#?}", options.view.mode);

            // List the current directory by default, like ls.
            // This has to be done here, otherwise git_options won’t see it.
            if args.is_empty() {
                args = vec![ OsStr::new(".") ];
            }

            let git = git_options(&options, &args);
            Exa { options, writer, args, git }
        })
    }

    pub fn run(&mut self) -> IOResult<i32> {
        let mut files = Vec::new();
        let mut dirs = Vec::new();
        let mut exit_status = 0;

        for file_path in &self.args {
            match File::new(PathBuf::from(file_path), None, None) {
                Err(e) => {
                    exit_status = 2;
                    writeln!(stderr(), "{:?}: {}", file_path, e)?;
                },
                Ok(f) => {
                    if f.is_directory() && !self.options.dir_action.treat_dirs_as_files() {
                        match f.to_dir() {
                            Ok(d) => dirs.push(d),
                            Err(e) => writeln!(stderr(), "{:?}: {}", file_path, e)?,
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
            for file in dir.files(self.options.filter.dot_filter) {
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
                        match child_dir.to_dir() {
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
            let View { ref mode, ref colours, ref style } = self.options.view;

            match *mode {
                Mode::Lines => {
                    lines::Render { files, colours, style }.render(self.writer)
                }
                Mode::Grid(ref opts) => {
                    grid::Render { files, colours, style, opts }.render(self.writer)
                }
                Mode::Details(ref opts) => {
                    details::Render { dir, files, colours, style, opts, filter: &self.options.filter, recurse: self.options.dir_action.recurse_options() }.render(self.git.as_ref(), self.writer)
                }
                Mode::GridDetails(ref opts) => {
                    grid_details::Render { dir, files, colours, style, grid: &opts.grid, details: &opts.details, filter: &self.options.filter, row_threshold: opts.row_threshold }.render(self.git.as_ref(), self.writer)
                }
            }
        }
        else {
            Ok(())
        }
    }
}
