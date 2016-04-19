extern crate exa;
use exa::Exa;

use std::env::args;
use std::io::stdout;
use std::process::exit;

fn main() {
    let args: Vec<String> = args().skip(1).collect();
    let mut stdout = stdout();

    match Exa::new(&args, &mut stdout) {
        Ok(mut exa) => exa.run().expect("IO error"),
        Err(e) => {
            println!("{}", e);
            exit(e.error_code());
        },
    };
}
