extern crate exa;
use exa::Exa;

use std::env::args_os;
use std::io::{stdout, stderr, Write, ErrorKind};
use std::process::exit;


fn main() {
    let args = args_os().skip(1);
    match Exa::new(args, &mut stdout()) {
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
            writeln!(stderr(), "{}", e).unwrap();
            exit(exits::OPTIONS_ERROR);
        },

        Err(ref e) => {
            writeln!(stdout(), "{}", e).unwrap();
            exit(exits::SUCCESS);
        },
    };
}


extern crate libc;
#[allow(trivial_numeric_casts)]
mod exits {
    use libc::{self, c_int};

    pub const SUCCESS:       c_int = libc::EXIT_SUCCESS;
    pub const RUNTIME_ERROR: c_int = libc::EXIT_FAILURE;
    pub const OPTIONS_ERROR: c_int = 3 as c_int;
}
