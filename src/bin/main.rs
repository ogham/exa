extern crate exa;
use exa::Exa;

use std::ffi::OsString;
use std::env::{args_os, var_os};
use std::io::{stdout, stderr, Write, ErrorKind};
use std::process::exit;


fn main() {
    configure_logger();

    let args: Vec<OsString> = args_os().skip(1).collect();
    match Exa::new(args.iter(), &mut stdout()) {
        Ok(mut exa) => {
            match exa.run() {
                Ok(exit_status) => exit(exit_status),
                Err(e) => {
                    match e.kind() {
                        ErrorKind::BrokenPipe => exit(exits::SUCCESS),
                        _ => {
                            writeln!(stderr(), "{}", e).unwrap();
                            exit(exits::RUNTIME_ERROR);
                        },
                    };
                }
            };
        },

        Err(ref e) if e.is_error() => {
            let mut stderr = stderr();
            writeln!(stderr, "{}", e).unwrap();

            if let Some(s) = e.suggestion() {
                let _ = writeln!(stderr, "{}", s);
            }

            exit(exits::OPTIONS_ERROR);
        },

        Err(ref e) => {
            writeln!(stdout(), "{}", e).unwrap();
            exit(exits::SUCCESS);
        },
    };
}


/// Sets up a global logger if one is asked for.
/// The ‘EXA_DEBUG’ environment variable controls whether log messages are
/// displayed or not. Currently there are just two settings (on and off).
///
/// This can’t be done in exa’s own option parsing because that part of it
/// logs as well, so by the time execution gets there, the logger needs to
/// have already been set up.
pub fn configure_logger() {
    extern crate env_logger;
    extern crate log;

    let present = match var_os(exa::vars::EXA_DEBUG) {
        Some(debug)  => debug.len() > 0,
        None         => false,
    };

    let mut logs = env_logger::LogBuilder::new();
    if present {
        logs.filter(None, log::LogLevelFilter::Debug);
    }
    else {
        logs.filter(None, log::LogLevelFilter::Off);
    }

    if let Err(e) = logs.init() {
        writeln!(stderr(), "Failed to initialise logger: {}", e).unwrap();
    }
}


extern crate libc;
#[allow(trivial_numeric_casts)]
mod exits {
    use libc::{self, c_int};

    pub const SUCCESS:       c_int = libc::EXIT_SUCCESS;
    pub const RUNTIME_ERROR: c_int = libc::EXIT_FAILURE;
    pub const OPTIONS_ERROR: c_int = 3 as c_int;
}
