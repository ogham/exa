use std::fmt;

use options::flags;
use options::parser::MatchedFlags;


/// All the information needed to display the version information.
#[derive(PartialEq, Debug)]
pub struct VersionString {

    /// The version number from cargo.
    cargo: &'static str,
}

impl VersionString {

    /// Determines how to show the version, if at all, based on the user’s
    /// command-line arguments. This one works backwards from the other
    /// ‘deduce’ functions, returning Err if help needs to be shown.
    ///
    /// Like --help, this doesn’t bother checking for errors.
    pub fn deduce(matches: &MatchedFlags) -> Result<(), VersionString> {
        if matches.count(&flags::VERSION) > 0 {
            Err(VersionString { cargo: env!("CARGO_PKG_VERSION") })
        }
        else {
            Ok(())  // no version needs to be shown
        }
    }
}

impl fmt::Display for VersionString {

    /// Format this help options into an actual string of help
    /// text to be displayed to the user.
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "exa v{}", self.cargo)
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
