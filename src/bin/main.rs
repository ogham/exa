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
                        ErrorKind::BrokenPipe => exit(0),
                        _ => {
                            writeln!(stderr(), "{}", e).unwrap();
                            exit(1);
                        },
                    };
                }
            };
        },

        Err(ref e) if e.is_error() => {
            writeln!(stderr(), "{}", e).unwrap();
            exit(3);
        },

        Err(ref e) => {
            writeln!(stdout(), "{}", e).unwrap();
            exit(0);
        },
    };
}
