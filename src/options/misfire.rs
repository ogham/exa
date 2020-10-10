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
        ! matches!(self, Self::Help(_) | Self::Version(_))
    }
}

impl From<glob::PatternError> for Misfire {
    fn from(error: glob::PatternError) -> Self {
        Self::FailedGlobPattern(error.to_string())
    }
}

impl fmt::Display for Misfire {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::options::parser::TakesValue;

        match self {
            Self::BadArgument(arg, attempt) => {
                if let TakesValue::Necessary(Some(values)) = arg.takes_value {
                    write!(f, "Option {} has no {:?} setting ({})", arg, attempt, Choices(values))
                }
                else {
                    write!(f, "Option {} has no {:?} setting", arg, attempt)
                }
            },
            Self::InvalidOptions(e)         => write!(f, "{}", e),
            Self::Unsupported(e)            => write!(f, "{}", e),
            Self::Help(text)                => write!(f, "{}", text),
            Self::Version(version)          => write!(f, "{}", version),
            Self::Conflict(a, b)            => write!(f, "Option {} conflicts with option {}", a, b),
            Self::Duplicate(a, b) if a == b => write!(f, "Flag {} was given twice", a),
            Self::Duplicate(a, b)           => write!(f, "Flag {} conflicts with flag {}", a, b),
            Self::Useless(a, false, b)      => write!(f, "Option {} is useless without option {}", a, b),
            Self::Useless(a, true, b)       => write!(f, "Option {} is useless given option {}", a, b),
            Self::Useless2(a, b1, b2)       => write!(f, "Option {} is useless without options {} or {}", a, b1, b2),
            Self::TreeAllAll                => write!(f, "Option --tree is useless given --all --all"),
            Self::FailedParse(ref e)        => write!(f, "Failed to parse number: {}", e),
            Self::FailedGlobPattern(ref e)  => write!(f, "Failed to parse glob pattern: {}", e),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NeedsValue { flag, values: None }     => write!(f, "Flag {} needs a value", flag),
            Self::NeedsValue { flag, values: Some(cs) } => write!(f, "Flag {} needs a value ({})", flag, Choices(cs)),
            Self::ForbiddenValue { flag }               => write!(f, "Flag {} cannot take a value", flag),
            Self::UnknownShortArgument { attempt }      => write!(f, "Unknown argument -{}", *attempt as char),
            Self::UnknownArgument { attempt }           => write!(f, "Unknown argument --{}", attempt.to_string_lossy()),
        }
    }
}

impl Misfire {
    /// Try to second-guess what the user was trying to do, depending on what
    /// went wrong.
    pub fn suggestion(&self) -> Option<&'static str> {
        // ‘ls -lt’ and ‘ls -ltr’ are common combinations
        match self {
            Self::BadArgument(time, r) if *time == &flags::TIME && r == "r" => {
                Some("To sort oldest files last, try \"--sort oldest\", or just \"-sold\"")
            }
            Self::InvalidOptions(ParseError::NeedsValue { ref flag, .. }) if *flag == Flag::Short(b't') => {
                Some("To sort newest files last, try \"--sort newest\", or just \"-snew\"")
            }
            _ => {
                None
            }
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
