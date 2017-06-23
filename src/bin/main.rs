extern crate exa;
use exa::{Exa, Misfire};

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
                        ErrorKind::BrokenPipe => exit(0),
                        _ => {
                            writeln!(stderr(), "{}", e).unwrap();
                            exit(1);
                        },
                    };
                }
            };
        },

        Err(e@Misfire::Help(_)) | Err(e@Misfire::Version) => {
            writeln!(stdout(), "{}", e).unwrap();
            exit(e.error_code());
        },

        Err(e) => {
            writeln!(stderr(), "{}", e).unwrap();
            exit(e.error_code());
        },
    };
}
