extern crate exa;
use exa::Exa;

use std::env::args;
use std::io::{stdout, stderr, Write, ErrorKind};
use std::process::exit;

fn main() {
    let args: Vec<String> = args().skip(1).collect();
    let mut stdout = stdout();

    match Exa::new(&args, &mut stdout) {
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
        Err(e) => {
            writeln!(stderr(), "{}", e).unwrap();
            exit(e.error_code());
        },
    };
}
