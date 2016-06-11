extern crate exa;
use exa::Exa;

/// --------------------------------------------------------------------------
/// These tests assume that the ‘generate annoying testcases’ script has been
/// run first. Otherwise, they will break!
/// --------------------------------------------------------------------------


static LINKS: &'static str = concat!(
    "\x1B[36m", "broken",  "\x1B[0m", " ", "\x1B[31m",       "->", "\x1B[0m", " ", "\x1B[4;31m", "testcases/links/nowhere", "\x1B[0m", '\n',
    "\x1B[36m", "root",    "\x1B[0m", " ", "\x1B[38;5;244m", "->", "\x1B[0m", " ", "\x1B[36m",   "/",                       "\x1B[0m", '\n',
    "\x1B[36m", "usr",     "\x1B[0m", " ", "\x1B[38;5;244m", "->", "\x1B[0m", " ", "\x1B[36m",   "/", "\x1B[1;34m", "usr",  "\x1B[0m", '\n',
);

#[test]
fn links() {
    let mut output = Vec::<u8>::new();
    Exa::new( &[ "-1", "testcases/links" ], &mut output).unwrap().run().unwrap();
    assert_eq!(output, LINKS.as_bytes());
}
