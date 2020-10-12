#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_results)]

use std::env;
use std::ffi::{OsStr, OsString};
use std::io::{stdout, Stdout, stderr, Write, Result as IOResult, ErrorKind};
use std::path::{Component, PathBuf};

use ansi_term::{ANSIStrings, Style};

use log::*;

use crate::fs::{Dir, File};
use crate::fs::feature::git::GitCache;
use crate::fs::filter::GitIgnore;
use crate::options::{Options, Vars, vars, OptionsResult};
use crate::output::{escape, lines, grid, grid_details, details, View, Mode};

mod fs;
mod info;
mod logger;
mod options;
mod output;
mod style;


fn main() {
    use std::process::exit;

    logger::configure(env::var_os(vars::EXA_DEBUG));

    let args: Vec<_> = env::args_os().skip(1).collect();
    match Options::parse(&args, &LiveVars) {
        OptionsResult::Ok(options, mut input_paths) => {

            // List the current directory by default.
            // (This has to be done here, otherwise git_options won’t see it.)
            if input_paths.is_empty() {
                input_paths = vec![ OsStr::new(".") ];
            }

            let git = git_options(&options, &input_paths);
            let writer = stdout();
            let exa = Exa { options, writer, input_paths, git };

            match exa.run() {
                Ok(exit_status) => {
                    exit(exit_status);
                }

                Err(e) if e.kind() == ErrorKind::BrokenPipe => {
                    warn!("Broken pipe error: {}", e);
                    exit(exits::SUCCESS);
                }

                Err(e) => {
                    eprintln!("{}", e);
                    exit(exits::RUNTIME_ERROR);
                }
            }
        }

        OptionsResult::Help(help_text) => {
            println!("{}", help_text);
        }

        OptionsResult::Version(version_str) => {
            println!("{}", version_str);
        }

        OptionsResult::InvalidOptions(error) => {
            eprintln!("{}", error);

            if let Some(s) = error.suggestion() {
                eprintln!("{}", s);
            }

            exit(exits::OPTIONS_ERROR);
        }
    }
}


/// The main program wrapper.
pub struct Exa<'args> {

    /// List of command-line options, having been successfully parsed.
    pub options: Options,

    /// The output handle that we write to.
    pub writer: Stdout,

    /// List of the free command-line arguments that should correspond to file
    /// names (anything that isn’t an option).
    pub input_paths: Vec<&'args OsStr>,

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
        env::var_os(name)
    }
}

/// Create a Git cache populated with the arguments that are going to be
/// listed before they’re actually listed, if the options demand it.
fn git_options(options: &Options, args: &[&OsStr]) -> Option<GitCache> {
    if options.should_scan_for_git() {
        Some(args.iter().map(PathBuf::from).collect())
    }
    else {
        None
    }
}

impl<'args> Exa<'args> {
    pub fn run(mut self) -> IOResult<i32> {
        debug!("Running with options: {:#?}", self.options);

        let mut files = Vec::new();
        let mut dirs = Vec::new();
        let mut exit_status = 0;

        for file_path in &self.input_paths {
            match File::from_args(PathBuf::from(file_path), None, None) {
                Err(e) => {
                    exit_status = 2;
                    writeln!(stderr(), "{:?}: {}", file_path, e)?;
                }

                Ok(f) => {
                    if f.points_to_directory() && ! self.options.dir_action.treat_dirs_as_files() {
                        match f.to_dir() {
                            Ok(d)   => dirs.push(d),
                            Err(e)  => writeln!(stderr(), "{:?}: {}", file_path, e)?,
                        }
                    }
                    else {
                        files.push(f);
                    }
                }
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
                writeln!(&mut self.writer)?;
            }

            if ! is_only_dir {
                let mut bits = Vec::new();
                escape(dir.path.display().to_string(), &mut bits, Style::default(), Style::default());
                writeln!(&mut self.writer, "{}:", ANSIStrings(&bits))?;
            }

            let mut children = Vec::new();
            let git_ignore = self.options.filter.git_ignore == GitIgnore::CheckAndIgnore;
            for file in dir.files(self.options.filter.dot_filter, self.git.as_ref(), git_ignore) {
                match file {
                    Ok(file)       => children.push(file),
                    Err((path, e)) => writeln!(stderr(), "[{}: {}]", path.display(), e)?,
                }
            };

            self.options.filter.filter_child_files(&mut children);
            self.options.filter.sort_files(&mut children);

            if let Some(recurse_opts) = self.options.dir_action.recurse_options() {
                let depth = dir.path.components().filter(|&c| c != Component::CurDir).count() + 1;
                if ! recurse_opts.tree && ! recurse_opts.is_too_deep(depth) {

                    let mut child_dirs = Vec::new();
                    for child_dir in children.iter().filter(|f| f.is_directory() && ! f.is_all_all) {
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
        if files.is_empty() {
            return Ok(());
        }

        let View { ref mode, ref colours, ref style } = self.options.view;

        match mode {
            Mode::Lines(ref opts) => {
                let r = lines::Render { files, colours, style, opts };
                r.render(&mut self.writer)
            }

            Mode::Grid(ref opts) => {
                let r = grid::Render { files, colours, style, opts };
                r.render(&mut self.writer)
            }

            Mode::Details(ref opts) => {
                let filter = &self.options.filter;
                let recurse = self.options.dir_action.recurse_options();

                let git_ignoring = self.options.filter.git_ignore == GitIgnore::CheckAndIgnore;
                let r = details::Render { dir, files, colours, style, opts, filter, recurse, git_ignoring };
                r.render(self.git.as_ref(), &mut self.writer)
            }

            Mode::GridDetails(ref opts) => {
                let grid = &opts.grid;
                let filter = &self.options.filter;
                let details = &opts.details;
                let row_threshold = opts.row_threshold;

                let git_ignoring = self.options.filter.git_ignore == GitIgnore::CheckAndIgnore;
                let r = grid_details::Render { dir, files, colours, style, grid, details, filter, row_threshold, git_ignoring };
                r.render(self.git.as_ref(), &mut self.writer)
            }
        }
    }
}


mod exits {

    /// Exit code for when exa runs OK.
    pub const SUCCESS: i32 = 0;

    /// Exit code for when there was at least one I/O error during execution.
    pub const RUNTIME_ERROR: i32 = 1;

    /// Exit code for when the command-line options are invalid.
    pub const OPTIONS_ERROR: i32 = 3;
}
