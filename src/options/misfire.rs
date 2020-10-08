use std::ffi::OsString;
use std::fmt;
use std::num::ParseIntError;

use crate::options::{flags, HelpString, VersionString};
use crate::options::parser::{Arg, Flag, ParseError};


/// A **misfire** is a thing that can happen instead of listing files -- a
/// catch-all for anything outside the program’s normal execution.
#[derive(PartialEq, Debug)]
pub enum Misfire {

    /// The getopts crate didn’t like these Arguments.
    InvalidOptions(ParseError),

    /// The user supplied an illegal choice to an Argument.
    BadArgument(&'static Arg, OsString),

    /// The user supplied a set of options
    Unsupported(String),

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
}

impl From<glob::PatternError> for Misfire {
    fn from(error: glob::PatternError) -> Misfire {
        Misfire::FailedGlobPattern(error.to_string())
    }
}

impl fmt::Display for Misfire {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::options::parser::TakesValue;
        use self::Misfire::*;

        match *self {
            BadArgument(ref arg, ref attempt) => {
                if let TakesValue::Necessary(Some(values)) = arg.takes_value {
                    write!(f, "Option {} has no {:?} setting ({})", arg, attempt, Choices(values))
                }
                else {
                    write!(f, "Option {} has no {:?} setting", arg, attempt)
                }
            },
            InvalidOptions(ref e)            => write!(f, "{}", e),
            Unsupported(ref e)               => write!(f, "{}", e),
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
            NeedsValue { ref flag, values: None }     => write!(f, "Flag {} needs a value", flag),
            NeedsValue { ref flag, values: Some(cs) } => write!(f, "Flag {} needs a value ({})", flag, Choices(cs)),
            ForbiddenValue { ref flag }               => write!(f, "Flag {} cannot take a value", flag),
            UnknownShortArgument { ref attempt }      => write!(f, "Unknown argument -{}", *attempt as char),
            UnknownArgument { ref attempt }           => write!(f, "Unknown argument --{}", attempt.to_string_lossy()),
        }
    }
}

impl Misfire {
    /// Try to second-guess what the user was trying to do, depending on what
    /// went wrong.
    pub fn suggestion(&self) -> Option<&'static str> {
        // ‘ls -lt’ and ‘ls -ltr’ are common combinations
        match *self {
            Misfire::BadArgument(ref time, ref r) if *time == &flags::TIME && r == "r" =>
                Some("To sort oldest files last, try \"--sort oldest\", or just \"-sold\""),
            Misfire::InvalidOptions(ParseError::NeedsValue { ref flag, .. }) if *flag == Flag::Short(b't') =>
                Some("To sort newest files last, try \"--sort newest\", or just \"-snew\""),
            _ => None
        }
    }
}


/// A list of legal choices for an argument-taking option.
#[derive(PartialEq, Debug)]
pub struct Choices(&'static [&'static str]);

impl fmt::Display for Choices {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "choices: {}", self.0.join(", "))
    }
}
