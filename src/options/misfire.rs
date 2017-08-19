use std::ffi::{OsStr, OsString};
use std::fmt;
use std::num::ParseIntError;

use glob;

use options::{HelpString, VersionString};
use options::parser::{Arg, Flag, ParseError};


/// A list of legal choices for an argument-taking option
#[derive(PartialEq, Debug)]
pub struct Choices(&'static [&'static str]);

impl fmt::Display for Choices {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(choices: {})", self.0.join(", "))
    }
}

/// A **misfire** is a thing that can happen instead of listing files -- a
/// catch-all for anything outside the program’s normal execution.
#[derive(PartialEq, Debug)]
pub enum Misfire {

    /// The getopts crate didn’t like these Arguments.
    InvalidOptions(ParseError),

    /// The user supplied an illegal choice to an Argument.
    BadArgument(&'static Arg, OsString, Choices),

    /// The user asked for help. This isn’t strictly an error, which is why
    /// this enum isn’t named Error!
    Help(HelpString),

    /// The user wanted the version number.
    Version(VersionString),

    /// An option was given twice or more in strict mode.
    Duplicate(Flag, Flag),

    /// Two options were given that conflict with one another.
    Conflict(&'static Arg, &'static Arg),

    /// An option was given that does nothing when another one either is or
    /// isn't present.
    Useless(&'static Arg, bool, &'static Arg),

    /// An option was given that does nothing when either of two other options
    /// are not present.
    Useless2(&'static Arg, &'static Arg, &'static Arg),

    /// A very specific edge case where --tree can’t be used with --all twice.
    TreeAllAll,

    /// A numeric option was given that failed to be parsed as a number.
    FailedParse(ParseIntError),

    /// A glob ignore was given that failed to be parsed as a pattern.
    FailedGlobPattern(String),
}

impl Misfire {

    /// The OS return code this misfire should signify.
    pub fn is_error(&self) -> bool {
        match *self {
            Misfire::Help(_)    => false,
            Misfire::Version(_) => false,
            _                   => true,
        }
    }

    /// The Misfire that happens when an option gets given the wrong
    /// argument. This has to use one of the `getopts` failure
    /// variants--it’s meant to take just an option name, rather than an
    /// option *and* an argument, but it works just as well.
    pub fn bad_argument(option: &'static Arg, otherwise: &OsStr, legal: &'static [&'static str]) -> Misfire {
        Misfire::BadArgument(option, otherwise.to_os_string(), Choices(legal))
    }
}

impl From<glob::PatternError> for Misfire {
    fn from(error: glob::PatternError) -> Misfire {
        Misfire::FailedGlobPattern(error.to_string())
    }
}

impl fmt::Display for Misfire {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Misfire::*;

        match *self {
            BadArgument(ref a, ref b, ref c) => write!(f, "Option {} has no value {:?} (Choices: {})", a, b, c),
            InvalidOptions(ref e)            => write!(f, "{}", e),
            Help(ref text)                   => write!(f, "{}", text),
            Version(ref version)             => write!(f, "{}", version),
            Conflict(ref a, ref b)           => write!(f, "Option {} conflicts with option {}", a, b),
            Duplicate(ref a, ref b)          => if a == b { write!(f, "Flag {} was given twice", a) }
                                                     else { write!(f, "Flag {} conflicts with flag {}", a, b) },
            Useless(ref a, false, ref b)     => write!(f, "Option {} is useless without option {}", a, b),
            Useless(ref a, true, ref b)      => write!(f, "Option {} is useless given option {}", a, b),
            Useless2(ref a, ref b1, ref b2)  => write!(f, "Option {} is useless without options {} or {}", a, b1, b2),
            TreeAllAll                       => write!(f, "Option --tree is useless given --all --all"),
            FailedParse(ref e)               => write!(f, "Failed to parse number: {}", e),
            FailedGlobPattern(ref e)         => write!(f, "Failed to parse glob pattern: {}", e),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ParseError::*;

        match *self {
            NeedsValue { ref flag }              => write!(f, "Flag {} needs a value", flag),
            ForbiddenValue { ref flag }          => write!(f, "Flag {} cannot take a value", flag),
            UnknownShortArgument { ref attempt } => write!(f, "Unknown argument -{}", *attempt as char),
            UnknownArgument { ref attempt }      => write!(f, "Unknown argument --{}", attempt.to_string_lossy()),
        }
    }
}
