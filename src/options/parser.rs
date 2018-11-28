//! A general parser for command-line options.
//!
//! exa uses its own hand-rolled parser for command-line options. It supports
//! the following syntax:
//!
//! - Long options: `--inode`, `--grid`
//! - Long options with values: `--sort size`, `--level=4`
//! - Short options: `-i`, `-G`
//! - Short options with values: `-ssize`, `-L=4`
//!
//! These values can be mixed and matched: `exa -lssize --grid`. If you’ve used
//! other command-line programs, then hopefully it’ll work much like them.
//!
//! Because exa already has its own files for the help text, shell completions,
//! man page, and readme, so it can get away with having the options parser do
//! very little: all it really needs to do is parse a slice of strings.
//!
//!
//! ## UTF-8 and `OsStr`
//!
//! The parser uses `OsStr` as its string type. This is necessary for exa to
//! list files that have invalid UTF-8 in their names: by treating file paths
//! as bytes with no encoding, a file can be specified on the command-line and
//! be looked up without having to be encoded into a `str` first.
//!
//! It also avoids the overhead of checking for invalid UTF-8 when parsing
//! command-line options, as all the options and their values (such as
//! `--sort size`) are guaranteed to just be 8-bit ASCII.

use std::ffi::{OsStr, OsString};
use std::fmt;

use options::Misfire;

/// A **short argument** is a single ASCII character.
pub type ShortArg = u8;

/// A **long argument** is a string. This can be a UTF-8 string, even though
/// the arguments will all be unchecked OsStrings, because we don’t actually
/// store the user’s input after it’s been matched to a flag, we just store
/// which flag it was.
pub type LongArg = &'static str;

/// A **list of values** that an option can have, to be displayed when the
/// user enters an invalid one or skips it.
///
/// This is literally just help text, and won’t be used to validate a value to
/// see if it’s correct.
pub type Values = &'static [&'static str];

/// A **flag** is either of the two argument types, because they have to
/// be in the same array together.
#[derive(PartialEq, Debug, Clone)]
pub enum Flag {
    Short(ShortArg),
    Long(LongArg),
}

impl Flag {
    pub fn matches(&self, arg: &Arg) -> bool {
        match *self {
            Flag::Short(short) => arg.short == Some(short),
            Flag::Long(long) => arg.long == long,
        }
    }
}

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Flag::Short(short) => write!(f, "-{}", short as char),
            Flag::Long(long) => write!(f, "--{}", long),
        }
    }
}

/// Whether redundant arguments should be considered a problem.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Strictness {
    /// Throw an error when an argument doesn’t do anything, either because
    /// it requires another argument to be specified, or because two conflict.
    ComplainAboutRedundantArguments,

    /// Search the arguments list back-to-front, giving ones specified later
    /// in the list priority over earlier ones.
    UseLastArguments,
}

/// Whether a flag takes a value. This is applicable to both long and short
/// arguments.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TakesValue {
    /// This flag has to be followed by a value.
    /// If there’s a fixed set of possible values, they can be printed out
    /// with the error text.
    Necessary(Option<Values>),

    /// This flag will throw an error if there’s a value after it.
    Forbidden,
}

/// An **argument** can be matched by one of the user’s input strings.
#[derive(PartialEq, Debug)]
pub struct Arg {
    /// The short argument that matches it, if any.
    pub short: Option<ShortArg>,

    /// The long argument that matches it. This is non-optional; all flags
    /// should at least have a descriptive long name.
    pub long: LongArg,

    /// Whether this flag takes a value or not.
    pub takes_value: TakesValue,
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "--{}", self.long)?;

        if let Some(short) = self.short {
            write!(f, " (-{})", short as char)?;
        }

        Ok(())
    }
}

/// Literally just several args.
#[derive(PartialEq, Debug)]
pub struct Args(pub &'static [&'static Arg]);

impl Args {
    /// Iterates over the given list of command-line arguments and parses
    /// them into a list of matched flags and free strings.
    pub fn parse<'args, I>(
        &self,
        inputs: I,
        strictness: Strictness,
    ) -> Result<Matches<'args>, ParseError>
    where
        I: IntoIterator<Item = &'args OsString>,
    {
        use self::TakesValue::*;
        use std::os::unix::ffi::OsStrExt;

        let mut parsing = true;

        // The results that get built up.
        let mut result_flags = Vec::new();
        let mut frees: Vec<&OsStr> = Vec::new();

        // Iterate over the inputs with “while let” because we need to advance
        // the iterator manually whenever an argument that takes a value
        // doesn’t have one in its string so it needs the next one.
        let mut inputs = inputs.into_iter();
        while let Some(arg) = inputs.next() {
            let bytes = arg.as_bytes();

            // Stop parsing if one of the arguments is the literal string “--”.
            // This allows a file named “--arg” to be specified by passing in
            // the pair “-- --arg”, without it getting matched as a flag that
            // doesn’t exist.
            if !parsing {
                frees.push(arg)
            } else if arg == "--" {
                parsing = false;
            }
            // If the string starts with *two* dashes then it’s a long argument.
            else if bytes.starts_with(b"--") {
                let long_arg_name = OsStr::from_bytes(&bytes[2..]);

                // If there’s an equals in it, then the string before the
                // equals will be the flag’s name, and the string after it
                // will be its value.
                if let Some((before, after)) = split_on_equals(long_arg_name) {
                    let arg = self.lookup_long(before)?;
                    let flag = Flag::Long(arg.long);
                    match arg.takes_value {
                        Necessary(_) => result_flags.push((flag, Some(after))),
                        Forbidden => return Err(ParseError::ForbiddenValue { flag }),
                    }
                }
                // If there’s no equals, then the entire string (apart from
                // the dashes) is the argument name.
                else {
                    let arg = self.lookup_long(long_arg_name)?;
                    let flag = Flag::Long(arg.long);
                    match arg.takes_value {
                        Forbidden => result_flags.push((flag, None)),
                        Necessary(values) => {
                            if let Some(next_arg) = inputs.next() {
                                result_flags.push((flag, Some(next_arg)));
                            } else {
                                return Err(ParseError::NeedsValue { flag, values });
                            }
                        }
                    }
                }
            }
            // If the string starts with *one* dash then it’s one or more
            // short arguments.
            else if bytes.starts_with(b"-") && arg != "-" {
                let short_arg = OsStr::from_bytes(&bytes[1..]);

                // If there’s an equals in it, then the argument immediately
                // before the equals was the one that has the value, with the
                // others (if any) as value-less short ones.
                //
                //   -x=abc         => ‘x=abc’
                //   -abcdx=fgh     => ‘a’, ‘b’, ‘c’, ‘d’, ‘x=fgh’
                //   -x=            =>  error
                //   -abcdx=        =>  error
                //
                // There’s no way to give two values in a cluster like this:
                // it’s an error if any of the first set of arguments actually
                // takes a value.
                if let Some((before, after)) = split_on_equals(short_arg) {
                    let (arg_with_value, other_args) = before.as_bytes().split_last().unwrap();

                    // Process the characters immediately following the dash...
                    for byte in other_args {
                        let arg = self.lookup_short(*byte)?;
                        let flag = Flag::Short(*byte);
                        match arg.takes_value {
                            Forbidden => result_flags.push((flag, None)),
                            Necessary(values) => {
                                return Err(ParseError::NeedsValue { flag, values })
                            }
                        }
                    }

                    // ...then the last one and the value after the equals.
                    let arg = self.lookup_short(*arg_with_value)?;
                    let flag = Flag::Short(arg.short.unwrap());
                    match arg.takes_value {
                        Necessary(_) => result_flags.push((flag, Some(after))),
                        Forbidden => return Err(ParseError::ForbiddenValue { flag }),
                    }
                }
                // If there’s no equals, then every character is parsed as
                // its own short argument. However, if any of the arguments
                // takes a value, then the *rest* of the string is used as
                // its value, and if there’s no rest of the string, then it
                // uses the next one in the iterator.
                //
                //   -a        => ‘a’
                //   -abc      => ‘a’, ‘b’, ‘c’
                //   -abxdef   => ‘a’, ‘b’, ‘x=def’
                //   -abx def  => ‘a’, ‘b’, ‘x=def’
                //   -abx      =>  error
                //
                else {
                    for (index, byte) in bytes.into_iter().enumerate().skip(1) {
                        let arg = self.lookup_short(*byte)?;
                        let flag = Flag::Short(*byte);
                        match arg.takes_value {
                            Forbidden => result_flags.push((flag, None)),
                            Necessary(values) => {
                                if index < bytes.len() - 1 {
                                    let remnants = &bytes[index + 1..];
                                    result_flags.push((flag, Some(OsStr::from_bytes(remnants))));
                                    break;
                                } else if let Some(next_arg) = inputs.next() {
                                    result_flags.push((flag, Some(next_arg)));
                                } else {
                                    return Err(ParseError::NeedsValue { flag, values });
                                }
                            }
                        }
                    }
                }
            }
            // Otherwise, it’s a free string, usually a file name.
            else {
                frees.push(arg)
            }
        }

        Ok(Matches {
            frees,
            flags: MatchedFlags {
                flags: result_flags,
                strictness,
            },
        })
    }

    fn lookup_short(&self, short: ShortArg) -> Result<&Arg, ParseError> {
        match self.0.into_iter().find(|arg| arg.short == Some(short)) {
            Some(arg) => Ok(arg),
            None => Err(ParseError::UnknownShortArgument { attempt: short }),
        }
    }

    fn lookup_long<'b>(&self, long: &'b OsStr) -> Result<&Arg, ParseError> {
        match self.0.into_iter().find(|arg| arg.long == long) {
            Some(arg) => Ok(arg),
            None => Err(ParseError::UnknownArgument {
                attempt: long.to_os_string(),
            }),
        }
    }
}

/// The **matches** are the result of parsing the user’s command-line strings.
#[derive(PartialEq, Debug)]
pub struct Matches<'args> {
    /// The flags that were parsed from the user’s input.
    pub flags: MatchedFlags<'args>,

    /// All the strings that weren’t matched as arguments, as well as anything
    /// after the special "--" string.
    pub frees: Vec<&'args OsStr>,
}

#[derive(PartialEq, Debug)]
pub struct MatchedFlags<'args> {
    /// The individual flags from the user’s input, in the order they were
    /// originally given.
    ///
    /// Long and short arguments need to be kept in the same vector because
    /// we usually want the one nearest the end to count, and to know this,
    /// we need to know where they are in relation to one another.
    flags: Vec<(Flag, Option<&'args OsStr>)>,

    /// Whether to check for duplicate or redundant arguments.
    strictness: Strictness,
}

impl<'a> MatchedFlags<'a> {
    /// Whether the given argument was specified.
    /// Returns `true` if it was, `false` if it wasn’t, and an error in
    /// strict mode if it was specified more than once.
    pub fn has(&self, arg: &'static Arg) -> Result<bool, Misfire> {
        self.has_where(|flag| flag.matches(arg))
            .map(|flag| flag.is_some())
    }

    /// Returns the first found argument that satisfies the predicate, or
    /// nothing if none is found, or an error in strict mode if multiple
    /// argument satisfy the predicate.
    ///
    /// You’ll have to test the resulting flag to see which argument it was.
    pub fn has_where<P>(&self, predicate: P) -> Result<Option<&Flag>, Misfire>
    where
        P: Fn(&Flag) -> bool,
    {
        if self.is_strict() {
            let all = self
                .flags
                .iter()
                .filter(|tuple| tuple.1.is_none() && predicate(&tuple.0))
                .collect::<Vec<_>>();

            if all.len() < 2 {
                Ok(all.first().map(|t| &t.0))
            } else {
                Err(Misfire::Duplicate(all[0].0.clone(), all[1].0.clone()))
            }
        } else {
            let any = self
                .flags
                .iter()
                .rev()
                .find(|tuple| tuple.1.is_none() && predicate(&tuple.0))
                .map(|tuple| &tuple.0);
            Ok(any)
        }
    }

    // This code could probably be better.
    // Both ‘has’ and ‘get’ immediately begin with a conditional, which makes
    // me think the functionality could be moved to inside Strictness.

    /// Returns the value of the given argument if it was specified, nothing
    /// if it wasn’t, and an error in strict mode if it was specified more
    /// than once.
    pub fn get(&self, arg: &'static Arg) -> Result<Option<&OsStr>, Misfire> {
        self.get_where(|flag| flag.matches(arg))
    }

    /// Returns the value of the argument that matches the predicate if it
    /// was specified, nothing if it wasn’t, and an error in strict mode if
    /// multiple arguments matched the predicate.
    ///
    /// It’s not possible to tell which flag the value belonged to from this.
    pub fn get_where<P>(&self, predicate: P) -> Result<Option<&OsStr>, Misfire>
    where
        P: Fn(&Flag) -> bool,
    {
        if self.is_strict() {
            let those = self
                .flags
                .iter()
                .filter(|tuple| tuple.1.is_some() && predicate(&tuple.0))
                .collect::<Vec<_>>();

            if those.len() < 2 {
                Ok(those.first().cloned().map(|t| t.1.unwrap()))
            } else {
                Err(Misfire::Duplicate(those[0].0.clone(), those[1].0.clone()))
            }
        } else {
            let found = self
                .flags
                .iter()
                .rev()
                .find(|tuple| tuple.1.is_some() && predicate(&tuple.0))
                .map(|tuple| tuple.1.unwrap());
            Ok(found)
        }
    }

    // It’s annoying that ‘has’ and ‘get’ won’t work when accidentally given
    // flags that do/don’t take values, but this should be caught by tests.

    /// Counts the number of occurrences of the given argument, even in
    /// strict mode.
    pub fn count(&self, arg: &Arg) -> usize {
        self.flags
            .iter()
            .filter(|tuple| tuple.0.matches(arg))
            .count()
    }

    /// Checks whether strict mode is on. This is usually done from within
    /// ‘has’ and ‘get’, but it’s available in an emergency.
    pub fn is_strict(&self) -> bool {
        self.strictness == Strictness::ComplainAboutRedundantArguments
    }
}

/// A problem with the user’s input that meant it couldn’t be parsed into a
/// coherent list of arguments.
#[derive(PartialEq, Debug)]
pub enum ParseError {
    /// A flag that has to take a value was not given one.
    NeedsValue { flag: Flag, values: Option<Values> },

    /// A flag that can’t take a value *was* given one.
    ForbiddenValue { flag: Flag },

    /// A short argument, either alone or in a cluster, was not
    /// recognised by the program.
    UnknownShortArgument { attempt: ShortArg },

    /// A long argument was not recognised by the program.
    /// We don’t have a known &str version of the flag, so
    /// this may not be valid UTF-8.
    UnknownArgument { attempt: OsString },
}

// It’s technically possible for ParseError::UnknownArgument to borrow its
// OsStr rather than owning it, but that would give ParseError a lifetime,
// which would give Misfire a lifetime, which gets used everywhere. And this
// only happens when an error occurs, so it’s not really worth it.

/// Splits a string on its `=` character, returning the two substrings on
/// either side. Returns `None` if there’s no equals or a string is missing.
fn split_on_equals(input: &OsStr) -> Option<(&OsStr, &OsStr)> {
    use std::os::unix::ffi::OsStrExt;

    if let Some(index) = input.as_bytes().iter().position(|elem| *elem == b'=') {
        let (before, after) = input.as_bytes().split_at(index);

        // The after string contains the = that we need to remove.
        if !before.is_empty() && after.len() >= 2 {
            return Some((OsStr::from_bytes(before), OsStr::from_bytes(&after[1..])));
        }
    }

    None
}

/// Creates an `OSString` (used in tests)
#[cfg(test)]
fn os(input: &'static str) -> OsString {
    let mut os = OsString::new();
    os.push(input);
    os
}

#[cfg(test)]
mod split_test {
    use super::{os, split_on_equals};

    macro_rules! test_split {
        ($name:ident: $input:expr => None) => {
            #[test]
            fn $name() {
                assert_eq!(split_on_equals(&os($input)), None);
            }
        };

        ($name:ident: $input:expr => $before:expr, $after:expr) => {
            #[test]
            fn $name() {
                assert_eq!(
                    split_on_equals(&os($input)),
                    Some((&*os($before), &*os($after)))
                );
            }
        };
    }

    test_split!(empty:   ""   => None);
    test_split!(letter:  "a"  => None);

    test_split!(just:      "="    => None);
    test_split!(intro:     "=bbb" => None);
    test_split!(denou:  "aaa="    => None);
    test_split!(equals: "aaa=bbb" => "aaa", "bbb");

    test_split!(sort: "--sort=size"     => "--sort", "size");
    test_split!(more: "this=that=other" => "this",   "that=other");
}

#[cfg(test)]
mod parse_test {
    use super::*;

    pub fn os(input: &'static str) -> OsString {
        let mut os = OsString::new();
        os.push(input);
        os
    }

    macro_rules! test {
        ($name:ident: $inputs:expr => frees: $frees:expr, flags: $flags:expr) => {
            #[test]
            fn $name() {
                // Annoyingly the input &strs need to be converted to OsStrings
                let inputs: Vec<OsString> = $inputs.as_ref().into_iter().map(|&o| os(o)).collect();

                // Same with the frees
                let frees: Vec<OsString> = $frees.as_ref().into_iter().map(|&o| os(o)).collect();
                let frees: Vec<&OsStr> = frees.iter().map(|os| os.as_os_str()).collect();

                let flags = <[_]>::into_vec(Box::new($flags));

                let strictness = Strictness::UseLastArguments; // this isn’t even used
                let got = Args(TEST_ARGS).parse(inputs.iter(), strictness);
                let expected = Ok(Matches {
                    frees,
                    flags: MatchedFlags { flags, strictness },
                });
                assert_eq!(got, expected);
            }
        };

        ($name:ident: $inputs:expr => error $error:expr) => {
            #[test]
            fn $name() {
                use self::ParseError::*;

                let strictness = Strictness::UseLastArguments; // this isn’t even used
                let bits = $inputs
                    .as_ref()
                    .into_iter()
                    .map(|&o| os(o))
                    .collect::<Vec<OsString>>();
                let got = Args(TEST_ARGS).parse(bits.iter(), strictness);

                assert_eq!(got, Err($error));
            }
        };
    }

    const SUGGESTIONS: Values = &["example"];

    static TEST_ARGS: &[&Arg] = &[
        &Arg {
            short: Some(b'l'),
            long: "long",
            takes_value: TakesValue::Forbidden,
        },
        &Arg {
            short: Some(b'v'),
            long: "verbose",
            takes_value: TakesValue::Forbidden,
        },
        &Arg {
            short: Some(b'c'),
            long: "count",
            takes_value: TakesValue::Necessary(None),
        },
        &Arg {
            short: Some(b't'),
            long: "type",
            takes_value: TakesValue::Necessary(Some(SUGGESTIONS)),
        },
    ];

    // Just filenames
    test!(empty:       []       => frees: [],         flags: []);
    test!(one_arg:     ["exa"]  => frees: [ "exa" ],  flags: []);

    // Dashes and double dashes
    test!(one_dash:    ["-"]             => frees: [ "-" ],       flags: []);
    test!(two_dashes:  ["--"]            => frees: [],            flags: []);
    test!(two_file:    ["--", "file"]    => frees: [ "file" ],    flags: []);
    test!(two_arg_l:   ["--", "--long"]  => frees: [ "--long" ],  flags: []);
    test!(two_arg_s:   ["--", "-l"]      => frees: [ "-l" ],      flags: []);

    // Long args
    test!(long:        ["--long"]               => frees: [],       flags: [ (Flag::Long("long"), None) ]);
    test!(long_then:   ["--long", "4"]          => frees: [ "4" ],  flags: [ (Flag::Long("long"), None) ]);
    test!(long_two:    ["--long", "--verbose"]  => frees: [],       flags: [ (Flag::Long("long"), None), (Flag::Long("verbose"), None) ]);

    // Long args with values
    test!(bad_equals:  ["--long=equals"]  => error ForbiddenValue { flag: Flag::Long("long") });
    test!(no_arg:      ["--count"]        => error NeedsValue     { flag: Flag::Long("count"), values: None });
    test!(arg_equals:  ["--count=4"]      => frees: [],  flags: [ (Flag::Long("count"), Some(OsStr::new("4"))) ]);
    test!(arg_then:    ["--count", "4"]   => frees: [],  flags: [ (Flag::Long("count"), Some(OsStr::new("4"))) ]);

    // Long args with values and suggestions
    test!(no_arg_s:      ["--type"]         => error NeedsValue { flag: Flag::Long("type"), values: Some(SUGGESTIONS) });
    test!(arg_equals_s:  ["--type=exa"]     => frees: [],  flags: [ (Flag::Long("type"), Some(OsStr::new("exa"))) ]);
    test!(arg_then_s:    ["--type", "exa"]  => frees: [],  flags: [ (Flag::Long("type"), Some(OsStr::new("exa"))) ]);

    // Short args
    test!(short:       ["-l"]            => frees: [],       flags: [ (Flag::Short(b'l'), None) ]);
    test!(short_then:  ["-l", "4"]       => frees: [ "4" ],  flags: [ (Flag::Short(b'l'), None) ]);
    test!(short_two:   ["-lv"]           => frees: [],       flags: [ (Flag::Short(b'l'), None), (Flag::Short(b'v'), None) ]);
    test!(mixed:       ["-v", "--long"]  => frees: [],       flags: [ (Flag::Short(b'v'), None), (Flag::Long("long"), None) ]);

    // Short args with values
    test!(bad_short:          ["-l=equals"]   => error ForbiddenValue { flag: Flag::Short(b'l') });
    test!(short_none:         ["-c"]          => error NeedsValue     { flag: Flag::Short(b'c'), values: None });
    test!(short_arg_eq:       ["-c=4"]        => frees: [],  flags: [(Flag::Short(b'c'), Some(OsStr::new("4"))) ]);
    test!(short_arg_then:     ["-c", "4"]     => frees: [],  flags: [(Flag::Short(b'c'), Some(OsStr::new("4"))) ]);
    test!(short_two_together: ["-lctwo"]      => frees: [],  flags: [(Flag::Short(b'l'), None), (Flag::Short(b'c'), Some(OsStr::new("two"))) ]);
    test!(short_two_equals:   ["-lc=two"]     => frees: [],  flags: [(Flag::Short(b'l'), None), (Flag::Short(b'c'), Some(OsStr::new("two"))) ]);
    test!(short_two_next:     ["-lc", "two"]  => frees: [],  flags: [(Flag::Short(b'l'), None), (Flag::Short(b'c'), Some(OsStr::new("two"))) ]);

    // Short args with values and suggestions
    test!(short_none_s:         ["-t"]         => error NeedsValue { flag: Flag::Short(b't'), values: Some(SUGGESTIONS) });
    test!(short_two_together_s: ["-texa"]      => frees: [],  flags: [(Flag::Short(b't'), Some(OsStr::new("exa"))) ]);
    test!(short_two_equals_s:   ["-t=exa"]     => frees: [],  flags: [(Flag::Short(b't'), Some(OsStr::new("exa"))) ]);
    test!(short_two_next_s:     ["-t", "exa"]  => frees: [],  flags: [(Flag::Short(b't'), Some(OsStr::new("exa"))) ]);

    // Unknown args
    test!(unknown_long:          ["--quiet"]      => error UnknownArgument      { attempt: os("quiet") });
    test!(unknown_long_eq:       ["--quiet=shhh"] => error UnknownArgument      { attempt: os("quiet") });
    test!(unknown_short:         ["-q"]           => error UnknownShortArgument { attempt: b'q' });
    test!(unknown_short_2nd:     ["-lq"]          => error UnknownShortArgument { attempt: b'q' });
    test!(unknown_short_eq:      ["-q=shhh"]      => error UnknownShortArgument { attempt: b'q' });
    test!(unknown_short_2nd_eq:  ["-lq=shhh"]     => error UnknownShortArgument { attempt: b'q' });
}

#[cfg(test)]
mod matches_test {
    use super::*;

    macro_rules! test {
        ($name:ident: $input:expr, has $param:expr => $result:expr) => {
            #[test]
            fn $name() {
                let flags = MatchedFlags {
                    flags: $input.to_vec(),
                    strictness: Strictness::UseLastArguments,
                };

                assert_eq!(flags.has(&$param), Ok($result));
            }
        };
    }

    static VERBOSE: Arg = Arg {
        short: Some(b'v'),
        long: "verbose",
        takes_value: TakesValue::Forbidden,
    };
    static COUNT: Arg = Arg {
        short: Some(b'c'),
        long: "count",
        takes_value: TakesValue::Necessary(None),
    };

    test!(short_never:  [],                                                              has VERBOSE => false);
    test!(short_once:   [(Flag::Short(b'v'), None)],                                     has VERBOSE => true);
    test!(short_twice:  [(Flag::Short(b'v'), None), (Flag::Short(b'v'), None)],          has VERBOSE => true);
    test!(long_once:    [(Flag::Long("verbose"), None)],                                 has VERBOSE => true);
    test!(long_twice:   [(Flag::Long("verbose"), None), (Flag::Long("verbose"), None)],  has VERBOSE => true);
    test!(long_mixed:   [(Flag::Long("verbose"), None), (Flag::Short(b'v'), None)],      has VERBOSE => true);

    #[test]
    fn only_count() {
        let everything = os("everything");

        let flags = MatchedFlags {
            flags: vec![(Flag::Short(b'c'), Some(&*everything))],
            strictness: Strictness::UseLastArguments,
        };

        assert_eq!(flags.get(&COUNT), Ok(Some(&*everything)));
    }

    #[test]
    fn rightmost_count() {
        let everything = os("everything");
        let nothing = os("nothing");

        let flags = MatchedFlags {
            flags: vec![
                (Flag::Short(b'c'), Some(&*everything)),
                (Flag::Short(b'c'), Some(&*nothing)),
            ],
            strictness: Strictness::UseLastArguments,
        };

        assert_eq!(flags.get(&COUNT), Ok(Some(&*nothing)));
    }

    #[test]
    fn no_count() {
        let flags = MatchedFlags {
            flags: Vec::new(),
            strictness: Strictness::UseLastArguments,
        };

        assert!(!flags.has(&COUNT).unwrap());
    }
}
