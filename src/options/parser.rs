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


pub type ShortArg = u8;
pub type LongArg = &'static str;


#[derive(PartialEq, Debug, Clone)]
pub enum Flag {
    Short(ShortArg),
    Long(LongArg),
}

impl Flag {
    fn matches(&self, arg: &Arg) -> bool {
        match *self {
            Flag::Short(short)  => arg.short == Some(short),
            Flag::Long(long)    => arg.long == long,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Strictness {
    ComplainAboutRedundantArguments,
    UseLastArguments,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TakesValue {
    Necessary,
    Forbidden,
}

#[derive(PartialEq, Debug)]
pub struct Arg {
    pub short: Option<ShortArg>,
    pub long: LongArg,
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

#[derive(PartialEq, Debug)]
pub struct Args(pub &'static [&'static Arg]);

impl Args {
    fn lookup_short<'a>(&self, short: ShortArg) -> Result<&Arg, ParseError> {
        match self.0.into_iter().find(|arg| arg.short == Some(short)) {
            Some(arg)  => Ok(arg),
            None       => Err(ParseError::UnknownShortArgument { attempt: short })
        }
    }

    fn lookup_long<'a>(&self, long: &'a OsStr) -> Result<&Arg, ParseError> {
        match self.0.into_iter().find(|arg| arg.long == long) {
            Some(arg)  => Ok(arg),
            None       => Err(ParseError::UnknownArgument { attempt: long.to_os_string() })
        }
    }
}


#[derive(PartialEq, Debug)]
pub struct Matches<'args> {
    /// Long and short arguments need to be kept in the same vector, because
    /// we usually want the one nearest the end to count.
    pub flags: Vec<(Flag, Option<&'args OsStr>)>,
    pub frees: Vec<&'args OsStr>,
}

impl<'a> Matches<'a> {
    pub fn has(&self, arg: &Arg) -> bool {
        self.flags.iter().rev()
            .find(|tuple| tuple.1.is_none() && tuple.0.matches(arg))
            .is_some()
    }

    pub fn get(&self, arg: &Arg) -> Option<&OsStr> {
        self.flags.iter().rev()
            .find(|tuple| tuple.1.is_some() && tuple.0.matches(arg))
            .map(|tuple| tuple.1.unwrap())
    }

    pub fn count(&self, arg: &Arg) -> usize {
        self.flags.iter()
            .filter(|tuple| tuple.0.matches(arg))
            .count()
    }
}

#[derive(PartialEq, Debug)]
pub enum ParseError {
    NeedsValue { flag: Flag },
    ForbiddenValue { flag: Flag },
    UnknownShortArgument { attempt: ShortArg },
    UnknownArgument { attempt: OsString },
}

// It’s technically possible for ParseError::UnknownArgument to borrow its
// OsStr rather than owning it, but that would give ParseError a lifetime,
// which would give Misfire a lifetime, which gets used everywhere. And this
// only happens when an error occurs, so it’s not really worth it.


pub fn parse<'args, I>(args: &Args, inputs: I) -> Result<Matches<'args>, ParseError>
where I: IntoIterator<Item=&'args OsString> {
    use std::os::unix::ffi::OsStrExt;
    use self::TakesValue::*;

    let mut parsing = true;

    let mut results = Matches {
        flags: Vec::new(),
        frees: Vec::new(),
    };

    let mut inputs = inputs.into_iter();
    while let Some(arg) = inputs.next() {
        let bytes = arg.as_bytes();

        if !parsing {
            results.frees.push(arg)
        }
        else if arg == "--" {
            parsing = false;
        }
        else if bytes.starts_with(b"--") {
            let long_arg_name = OsStr::from_bytes(&bytes[2..]);

            if let Some((before, after)) = split_on_equals(long_arg_name) {
                let arg = args.lookup_long(before)?;
                let flag = Flag::Long(arg.long);
                match arg.takes_value {
                    Necessary  => results.flags.push((flag, Some(after))),
                    Forbidden  => return Err(ParseError::ForbiddenValue { flag })
                }
            }
            else {
                let arg = args.lookup_long(long_arg_name)?;
                let flag = Flag::Long(arg.long);
                match arg.takes_value {
                    Forbidden  => results.flags.push((flag, None)),
                    Necessary  => {
                        if let Some(next_arg) = inputs.next() {
                            results.flags.push((flag, Some(next_arg)));
                        }
                        else {
                            return Err(ParseError::NeedsValue { flag })
                        }
                    }
                }
            }
        }
        else if bytes.starts_with(b"-") && arg != "-" {
            let short_arg = OsStr::from_bytes(&bytes[1..]);
            if let Some((before, after)) = split_on_equals(short_arg) {
                let (arg_with_value, other_args) = before.as_bytes().split_last().unwrap();

                for byte in other_args {
                    let arg = args.lookup_short(*byte)?;
                    let flag = Flag::Short(*byte);
                    match arg.takes_value {
                        Forbidden  => results.flags.push((flag, None)),
                        Necessary  => return Err(ParseError::NeedsValue { flag })
                    }
                }

                let arg = args.lookup_short(*arg_with_value)?;
                let flag = Flag::Short(arg.short.unwrap());
                match arg.takes_value {
                    Necessary  => results.flags.push((flag, Some(after))),
                    Forbidden  => return Err(ParseError::ForbiddenValue { flag })
                }
            }
            else {
                for (index, byte) in bytes.into_iter().enumerate().skip(1) {
                    let arg = args.lookup_short(*byte)?;
                    let flag = Flag::Short(*byte);
                    match arg.takes_value {
                        Forbidden  => results.flags.push((flag, None)),
                        Necessary  => {
                            if index < bytes.len() - 1 {
                                let remnants = &bytes[index+1 ..];
                                results.flags.push((flag, Some(OsStr::from_bytes(remnants))));
                                break;
                            }
                            else if let Some(next_arg) = inputs.next() {
                                results.flags.push((flag, Some(next_arg)));
                            }
                            else {
                                return Err(ParseError::NeedsValue { flag })
                            }
                        }
                    }
                }
            }
        }
        else {
            results.frees.push(arg)
        }
    }

    Ok(results)
}


/// Splits a string on its `=` character, returning the two substrings on
/// either side. Returns `None` if there’s no equals or a string is missing.
fn split_on_equals(input: &OsStr) -> Option<(&OsStr, &OsStr)> {
    use std::os::unix::ffi::OsStrExt;

    if let Some(index) = input.as_bytes().iter().position(|elem| *elem == b'=') {
        let (before, after) = input.as_bytes().split_at(index);

        // The after string contains the = that we need to remove.
        if before.len() >= 1 && after.len() >= 2 {
            return Some((OsStr::from_bytes(before),
                         OsStr::from_bytes(&after[1..])))
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
    use super::{split_on_equals, os};

    macro_rules! test_split {
        ($name:ident: $input:expr => None) => {
            #[test]
            fn $name() {
                assert_eq!(split_on_equals(&os($input)),
                           None);
            }
        };

        ($name:ident: $input:expr => $before:expr, $after:expr) => {
            #[test]
            fn $name() {
                assert_eq!(split_on_equals(&os($input)),
                           Some((&*os($before), &*os($after))));
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

    macro_rules! test {
        ($name:ident: $inputs:expr => $result:expr) => {
            #[test]
            fn $name() {
                let bits = $inputs.as_ref().into_iter().map(|&o| os(o)).collect::<Vec<OsString>>();
                let results = parse(&Args(TEST_ARGS), bits.iter());
                assert_eq!(results, $result);
            }
        };
    }

    static TEST_ARGS: &[&Arg] = &[
        &Arg { short: Some(b'l'), long: "long",     takes_value: TakesValue::Forbidden },
        &Arg { short: Some(b'v'), long: "verbose",  takes_value: TakesValue::Forbidden },
        &Arg { short: Some(b'c'), long: "count",    takes_value: TakesValue::Necessary }
    ];


    // Just filenames
    test!(empty:       []       => Ok(Matches { frees: vec![],             flags: vec![] }));
    test!(one_arg:     ["exa"]  => Ok(Matches { frees: vec![ &os("exa") ], flags: vec![] }));

    // Dashes and double dashes
    test!(one_dash:    ["-"]             => Ok(Matches { frees: vec![ &os("-") ],      flags: vec![] }));
    test!(two_dashes:  ["--"]            => Ok(Matches { frees: vec![],                flags: vec![] }));
    test!(two_file:    ["--", "file"]    => Ok(Matches { frees: vec![ &os("file") ],   flags: vec![] }));
    test!(two_arg_l:   ["--", "--long"]  => Ok(Matches { frees: vec![ &os("--long") ], flags: vec![] }));
    test!(two_arg_s:   ["--", "-l"]      => Ok(Matches { frees: vec![ &os("-l") ],     flags: vec![] }));


    // Long args
    test!(long:        ["--long"]               => Ok(Matches { frees: vec![],           flags: vec![ (Flag::Long("long"), None) ] }));
    test!(long_then:   ["--long", "4"]          => Ok(Matches { frees: vec![ &os("4") ], flags: vec![ (Flag::Long("long"), None) ] }));
    test!(long_two:    ["--long", "--verbose"]  => Ok(Matches { frees: vec![],           flags: vec![ (Flag::Long("long"), None), (Flag::Long("verbose"), None) ] }));

    // Long args with values
    test!(bad_equals:  ["--long=equals"]  => Err(ParseError::ForbiddenValue { flag: Flag::Long("long") }));
    test!(no_arg:      ["--count"]        => Err(ParseError::NeedsValue     { flag: Flag::Long("count") }));
    test!(arg_equals:  ["--count=4"]      => Ok(Matches { frees: vec![], flags: vec![ (Flag::Long("count"), Some(&*os("4"))) ] }));
    test!(arg_then:    ["--count", "4"]   => Ok(Matches { frees: vec![], flags: vec![ (Flag::Long("count"), Some(&*os("4"))) ] }));


    // Short args
    test!(short:       ["-l"]            => Ok(Matches { frees: vec![],            flags: vec![ (Flag::Short(b'l'), None) ] }));
    test!(short_then:  ["-l", "4"]       => Ok(Matches { frees: vec![ &*os("4") ], flags: vec![ (Flag::Short(b'l'), None) ] }));
    test!(short_two:   ["-lv"]           => Ok(Matches { frees: vec![],            flags: vec![ (Flag::Short(b'l'), None), (Flag::Short(b'v'), None) ] }));
    test!(mixed:       ["-v", "--long"]  => Ok(Matches { frees: vec![],            flags: vec![ (Flag::Short(b'v'), None), (Flag::Long("long"), None) ] }));

    // Short args with values
    test!(bad_short:          ["-l=equals"]   => Err(ParseError::ForbiddenValue { flag: Flag::Short(b'l') }));
    test!(short_none:         ["-c"]          => Err(ParseError::NeedsValue     { flag: Flag::Short(b'c') }));
    test!(short_arg_eq:       ["-c=4"]        => Ok(Matches { frees: vec![], flags: vec![ (Flag::Short(b'c'), Some(&*os("4"))) ] }));
    test!(short_arg_then:     ["-c", "4"]     => Ok(Matches { frees: vec![], flags: vec![ (Flag::Short(b'c'), Some(&*os("4"))) ] }));
    test!(short_two_together: ["-lctwo"]      => Ok(Matches { frees: vec![], flags: vec![ (Flag::Short(b'l'), None), (Flag::Short(b'c'), Some(&*os("two"))) ] }));
    test!(short_two_equals:   ["-lc=two"]     => Ok(Matches { frees: vec![], flags: vec![ (Flag::Short(b'l'), None), (Flag::Short(b'c'), Some(&*os("two"))) ] }));
    test!(short_two_next:     ["-lc", "two"]  => Ok(Matches { frees: vec![], flags: vec![ (Flag::Short(b'l'), None), (Flag::Short(b'c'), Some(&*os("two"))) ] }));


    // Unknown args
    test!(unknown_long:          ["--quiet"]      => Err(ParseError::UnknownArgument      { attempt: os("quiet") }));
    test!(unknown_long_eq:       ["--quiet=shhh"] => Err(ParseError::UnknownArgument      { attempt: os("quiet") }));
    test!(unknown_short:         ["-q"]           => Err(ParseError::UnknownShortArgument { attempt: b'q' }));
    test!(unknown_short_2nd:     ["-lq"]          => Err(ParseError::UnknownShortArgument { attempt: b'q' }));
    test!(unknown_short_eq:      ["-q=shhh"]      => Err(ParseError::UnknownShortArgument { attempt: b'q' }));
    test!(unknown_short_2nd_eq:  ["-lq=shhh"]     => Err(ParseError::UnknownShortArgument { attempt: b'q' }));
}


#[cfg(test)]
mod matches_test {
    use super::*;

    macro_rules! test {
        ($name:ident: $input:expr, has $param:expr => $result:expr) => {
            #[test]
            fn $name() {
                let frees = Vec::new();
                let flags = $input.to_vec();
                assert_eq!(Matches { frees, flags }.has(&$param), $result);
            }
        };
    }

    static VERBOSE: Arg = Arg { short: Some(b'v'), long: "verbose",  takes_value: TakesValue::Forbidden };
    static COUNT:   Arg = Arg { short: Some(b'c'), long: "count",    takes_value: TakesValue::Necessary };
    static TEST_ARGS: &[&Arg] = &[ &VERBOSE, &COUNT ];


    test!(short_never: [],                                                     has VERBOSE => false);
    test!(short_once:  [(Flag::Short(b'v'), None)],                            has VERBOSE => true);
    test!(short_twice: [(Flag::Short(b'v'), None), (Flag::Short(b'v'), None)], has VERBOSE => true);

    test!(long_once:  [(Flag::Long("verbose"), None)],                                has VERBOSE => true);
    test!(long_twice: [(Flag::Long("verbose"), None), (Flag::Long("verbose"), None)], has VERBOSE => true);
    test!(long_mixed: [(Flag::Long("verbose"), None), (Flag::Short(b'v'), None)],     has VERBOSE => true);


    #[test]
    fn only_count() {
        let everything = os("everything");
        let frees = Vec::new();
        let flags = vec![ (Flag::Short(b'c'), Some(&*everything)) ];
        assert_eq!(Matches { frees, flags }.get(&COUNT), Some(&*everything));
    }

    #[test]
    fn rightmost_count() {
        let everything = os("everything");
        let nothing    = os("nothing");

        let frees = Vec::new();
        let flags = vec![ (Flag::Short(b'c'), Some(&*everything)),
                          (Flag::Short(b'c'), Some(&*nothing)) ];

        assert_eq!(Matches { frees, flags }.get(&COUNT), Some(&*nothing));
    }

    #[test]
    fn no_count() {
        let frees =  Vec::new();
        let flags =  Vec::new();

        assert!(!Matches { frees, flags }.has(&COUNT));
    }
}
