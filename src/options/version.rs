//! Printing the version string.
//!
//! The code that works out which string to print is done in `build.rs`.

use std::fmt;

use crate::options::flags;
use crate::options::parser::MatchedFlags;


#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct VersionString;
// There were options here once, but there aren’t anymore!

impl VersionString {

    /// Determines how to show the version, if at all, based on the user’s
    /// command-line arguments. This one works backwards from the other
    /// ‘deduce’ functions, returning Err if help needs to be shown.
    ///
    /// Like --help, this doesn’t check for errors.
    pub fn deduce(matches: &MatchedFlags<'_>) -> Option<Self> {
        if matches.count(&flags::VERSION) > 0 {
            Some(Self)
        }
        else {
            None
        }
    }
}

impl fmt::Display for VersionString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", include_str!(concat!(env!("OUT_DIR"), "/version_string.txt")))
    }
}


#[cfg(test)]
mod test {
    use crate::options::{Options, OptionsResult};
    use std::ffi::OsStr;

    #[test]
    fn version() {
        let args = vec![ OsStr::new("--version") ];
        let opts = Options::parse(args, &None);
        assert!(matches!(opts, OptionsResult::Version(_)));
    }

    #[test]
    fn version_with_file() {
        let args = vec![ OsStr::new("--version"), OsStr::new("me") ];
        let opts = Options::parse(args, &None);
        assert!(matches!(opts, OptionsResult::Version(_)));
    }
}
