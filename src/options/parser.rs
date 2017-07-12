#![allow(unused_variables, dead_code)]

use std::ffi::{OsStr, OsString};


pub type ShortArg = u8;
pub type LongArg = &'static str;


#[derive(PartialEq, Debug)]
pub enum Flag {
    Short(ShortArg),
    Long(LongArg),
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
    short: Option<ShortArg>,
    long: LongArg,
    takes_value: TakesValue,
}

#[derive(PartialEq, Debug)]
pub struct Args(&'static [Arg]);

impl Args {
    fn lookup_short<'a>(&self, short: ShortArg) -> Result<&Arg, ParseError<'a>> {
        match self.0.into_iter().find(|arg| arg.short == Some(short)) {
            Some(arg)  => Ok(arg),
            None       => Err(ParseError::UnknownShortArgument { attempt: short })
        }
    }

    fn lookup_long<'a>(&self, long: &'a OsStr) -> Result<&Arg, ParseError<'a>> {
        match self.0.into_iter().find(|arg| arg.long == long) {
            Some(arg)  => Ok(arg),
            None       => Err(ParseError::UnknownArgument { attempt: long })
        }
    }
}


#[derive(PartialEq, Debug)]
pub struct Matches<'a> {
    /// Long and short arguments need to be kept in the same vector, because
    /// we usually want the one nearest the end to count.
    flags: Vec<(Flag, Option<&'a OsStr>)>,
    frees: Vec<&'a OsStr>,
}

#[derive(PartialEq, Debug)]
pub enum ParseError<'a> {
    NeedsValue { flag: Flag },
    ForbiddenValue { flag: Flag },
    UnknownShortArgument { attempt: ShortArg },
    UnknownArgument { attempt: &'a OsStr },
}


fn parse<'a>(args: Args, inputs: &'a [OsString]) -> Result<Matches<'a>, ParseError<'a>> {
    use std::os::unix::ffi::OsStrExt;
    use self::TakesValue::*;

    let mut parsing = true;

    let mut results = Matches {
        flags: Vec::new(),
        frees: Vec::new(),
    };

    let mut iter = inputs.iter();
    while let Some(arg) = iter.next() {
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
                        if let Some(next_arg) = iter.next() {
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

                let arg = args.lookup_short(*before.as_bytes().last().unwrap())?;
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
                            else if let Some(next_arg) = iter.next() {
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
mod test {
    use super::*;

    static TEST_ARGS: &'static [Arg] = &[
        Arg { short: Some(b'l'), long: "long",     takes_value: TakesValue::Forbidden },
        Arg { short: Some(b'v'), long: "verbose",  takes_value: TakesValue::Forbidden },
        Arg { short: Some(b'c'), long: "count",    takes_value: TakesValue::Necessary }
    ];

    #[test]
    fn empty() {
        let bits = [ ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![], flags: vec![] }))
    }

    #[test]
    fn filename() {
        let bits = [ os("exa") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![ os("exa").as_os_str() ], flags: vec![] }))
    }

    #[test]
    fn the_dashes_do_nothing() {
        let bits = [ os("--") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![], flags: vec![] }))
    }

    #[test]
    fn but_just_one_does() {
        let bits = [ os("-") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![ os("-").as_os_str() ], flags: vec![] }))
    }



    // ----- long args --------

    #[test]
    fn as_filename() {
        let bits = [ os("--"), os("--long") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![os("--long").as_os_str() ], flags: vec![] }))
    }


    #[test]
    fn long() {
        let bits = [ os("--long") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![], flags: vec![ (Flag::Long("long"), None) ] }))
    }

    #[test]
    fn long_equals() {
        let bits = [ os("--long=equals") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Err(ParseError::ForbiddenValue { flag: Flag::Long("long") }))
    }

    #[test]
    fn no_arg_separate() {
        let bits = [ os("--long"), os("4") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![ os("4").as_os_str() ], flags: vec![ (Flag::Long("long"), None) ] }))
    }

    #[test]
    fn two_long_flags() {
        let bits = [ os("--long"), os("--verbose") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![], flags: vec![ (Flag::Long("long"), None), (Flag::Long("verbose"), None) ] }))
    }

    #[test]
    fn no_arg_given() {
        let bits = [ os("--count") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Err(ParseError::NeedsValue { flag: Flag::Long("count") }))
    }

    #[test]
    fn arg_equals() {
        let bits = [ os("--count=4") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![], flags: vec![ (Flag::Long("count"), Some(os("4").as_os_str())) ] }))
    }

    #[test]
    fn arg_separate() {
        let bits = [ os("--count"), os("4") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![], flags: vec![ (Flag::Long("count"), Some(os("4").as_os_str())) ] }))
    }






    // ----- short args --------

    #[test]
    fn short_as_filename() {
        let bits = [ os("--"), os("-l") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![os("-l").as_os_str() ], flags: vec![] }))
    }


    #[test]
    fn short_long() {
        let bits = [ os("-l") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![], flags: vec![ (Flag::Short(b'l'), None) ] }))
    }

    #[test]
    fn short_long_equals() {
        let bits = [ os("-l=equals") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Err(ParseError::ForbiddenValue { flag: Flag::Short(b'l') }))
    }

    #[test]
    fn short_no_arg_separate() {
        let bits = [ os("-l"), os("4") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![ os("4").as_os_str() ], flags: vec![ (Flag::Short(b'l'), None) ] }))
    }


    #[test]
    fn short_no_arg_given() {
        let bits = [ os("-c") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Err(ParseError::NeedsValue { flag: Flag::Short(b'c') }))
    }

    #[test]
    fn short_arg_equals() {
        let bits = [ os("-c=4") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![], flags: vec![ (Flag::Short(b'c'), Some(os("4").as_os_str())) ] }))
    }

    #[test]
    fn short_arg_separate() {
        let bits = [ os("-c"), os("4") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![], flags: vec![ (Flag::Short(b'c'), Some(os("4").as_os_str())) ] }))
    }

    #[test]
    fn two_short_flags() {
        let bits = [ os("-lv") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![], flags: vec![ (Flag::Short(b'l'), None), (Flag::Short(b'v'), None) ] }))
    }

    #[test]
    fn two_short_flags_nothing() {
        let bits = [ os("-lctwo") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![], flags: vec![ (Flag::Short(b'l'), None), (Flag::Short(b'c'), Some(os("two").as_os_str())) ] }))
    }

    #[test]
    fn two_short_flags_equals() {
        let bits = [ os("-lc=two") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![], flags: vec![ (Flag::Short(b'l'), None), (Flag::Short(b'c'), Some(os("two").as_os_str())) ] }))
    }

    #[test]
    fn two_short_flags_next() {
        let bits = [ os("-lc"), os("two") ];
        let results = parse(Args(TEST_ARGS), &bits);
        assert_eq!(results, Ok(Matches { frees: vec![], flags: vec![ (Flag::Short(b'l'), None), (Flag::Short(b'c'), Some(os("two").as_os_str())) ] }))
    }
}
