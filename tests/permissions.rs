extern crate exa;
use exa::Exa;

/// --------------------------------------------------------------------------
/// These tests assume that the ‘generate annoying testcases’ script has been
/// run first. Otherwise, they will break!
/// --------------------------------------------------------------------------


static PERMISSIONS: &'static str = concat!(
    "\x1B[1;32m", "all-permissions",     "\x1B[0m", '\n',
    "\x1B[1;34m", "forbidden-directory", "\x1B[0m", '\n',
                  "no-permissions",                 '\n',
);

#[test]
fn permissions() {
    let mut output = Vec::<u8>::new();
    Exa::new( &[ "-1", "testcases/permissions" ], &mut output).unwrap().run().unwrap();
    assert_eq!(output, PERMISSIONS.as_bytes());
}
