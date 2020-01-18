//! Printing the version string.
//!
//! The code that works out which string to print is done in `build.rs`.

use std::fmt;

use options::flags;
use options::parser::MatchedFlags;


#[derive(PartialEq, Debug)]
pub struct VersionString;
// There were options here once, but there aren’t anymore!

impl VersionString {

    /// Determines how to show the version, if at all, based on the user’s
    /// command-line arguments. This one works backwards from the other
    /// ‘deduce’ functions, returning Err if help needs to be shown.
    ///
    /// Like --help, this doesn’t bother checking for errors.
    pub fn deduce(matches: &MatchedFlags) -> Result<(), VersionString> {
        if matches.count(&flags::VERSION) > 0 {
            Err(VersionString)
        }
        else {
            Ok(())  // no version needs to be shown
        }
    }
}

impl fmt::Display for VersionString {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", include!(concat!(env!("OUT_DIR"), "/version_string.txt")))
    }
}


#[cfg(test)]
mod test {
    use options::Options;
    use std::ffi::OsString;

    fn os(input: &'static str) -> OsString {
        let mut os = OsString::new();
        os.push(input);
        os
    }

    #[test]
    fn help() {
        let args = [ os("--version") ];
        let opts = Options::parse(&args, &None);
        assert!(opts.is_err())
    }
}
