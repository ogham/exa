use std::fmt;
use std::num::ParseIntError;

use getopts;
use glob;

use options::help::HelpString;


/// A list of legal choices for an argument-taking option
#[derive(PartialEq, Debug)]
pub struct Choices(&'static [&'static str]);

impl fmt::Display for Choices {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(choices: {})", self.0.join(" "))
    }
}

/// A **misfire** is a thing that can happen instead of listing files -- a
/// catch-all for anything outside the program’s normal execution.
#[derive(PartialEq, Debug)]
pub enum Misfire {

    /// The getopts crate didn’t like these arguments.
    InvalidOptions(getopts::Fail),

    /// The user supplied an illegal choice to an argument
    BadArgument(getopts::Fail, Choices),

    /// The user asked for help. This isn’t strictly an error, which is why
    /// this enum isn’t named Error!
    Help(HelpString),

    /// The user wanted the version number.
    Version,

    /// Two options were given that conflict with one another.
    Conflict(&'static str, &'static str),

    /// An option was given that does nothing when another one either is or
    /// isn't present.
    Useless(&'static str, bool, &'static str),

    /// An option was given that does nothing when either of two other options
    /// are not present.
    Useless2(&'static str, &'static str, &'static str),

    /// A numeric option was given that failed to be parsed as a number.
    FailedParse(ParseIntError),

    /// A glob ignore was given that failed to be parsed as a pattern.
    FailedGlobPattern(String),
}

impl Misfire {

    /// The OS return code this misfire should signify.
    pub fn is_error(&self) -> bool {
        match *self {
            Misfire::Help(_) => false,
            Misfire::Version => false,
            _                => true,
        }
    }

    /// The Misfire that happens when an option gets given the wrong
    /// argument. This has to use one of the `getopts` failure
    /// variants--it’s meant to take just an option name, rather than an
    /// option *and* an argument, but it works just as well.
    pub fn bad_argument(option: &str, otherwise: &str, legal: &'static [&'static str]) -> Misfire {
        Misfire::BadArgument(getopts::Fail::UnrecognizedOption(format!(
            "--{} {}",
            option, otherwise)), Choices(legal))
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
            InvalidOptions(ref e)      => write!(f, "{}", e),
            BadArgument(ref e, ref c)  => write!(f, "{} {}", e, c),
            Help(ref text)             => write!(f, "{}", text),
            Version                    => write!(f, "exa {}", env!("CARGO_PKG_VERSION")),
            Conflict(a, b)             => write!(f, "Option --{} conflicts with option {}.", a, b),
            Useless(a, false, b)       => write!(f, "Option --{} is useless without option --{}.", a, b),
            Useless(a, true, b)        => write!(f, "Option --{} is useless given option --{}.", a, b),
            Useless2(a, b1, b2)        => write!(f, "Option --{} is useless without options --{} or --{}.", a, b1, b2),
            FailedParse(ref e)         => write!(f, "Failed to parse number: {}", e),
            FailedGlobPattern(ref e)   => write!(f, "Failed to parse glob pattern: {}", e),
        }
    }
}
