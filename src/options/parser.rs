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
    fn lookup_short(&self, short: ShortArg) -> Option<&Arg> {
        self.0.into_iter().find(|arg| arg.short == Some(short))
    }

    fn lookup_long(&self, long: &OsStr) -> Option<&Arg> {
        self.0.into_iter().find(|arg| arg.long == long)
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
            let long_arg = OsStr::from_bytes(&bytes[2..]);

            if let Some((before, after)) = split_on_equals(long_arg) {
                if let Some(&Arg { short: _, long: long_arg_name, takes_value }) = args.lookup_long(before) {
                    let flag = Flag::Long(long_arg_name);
                    match takes_value {
                        Necessary  => results.flags.push((flag, Some(after))),
                        Forbidden  => return Err(ParseError::ForbiddenValue { flag })
                    }
                }
                else {
                    return Err(ParseError::UnknownArgument { attempt: before })
                }
            }
            else {
                if let Some(&Arg { short: _, long: long_arg_name, takes_value }) = args.lookup_long(long_arg) {
                    let flag = Flag::Long(long_arg_name);
                    match takes_value {
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
                else {
                    return Err(ParseError::UnknownArgument { attempt: long_arg })
                }
            }
        }
        else if bytes.starts_with(b"-") && arg != "-" {
            let short_arg = OsStr::from_bytes(&bytes[1..]);
            if let Some((before, after)) = split_on_equals(short_arg) {
                // TODO: remember to deal with the other bytes!
                if let Some(&Arg { short, long, takes_value }) = args.lookup_short(*before.as_bytes().last().unwrap()) {
                    let flag = Flag::Short(short.unwrap());
                    match takes_value {
                        Necessary  => results.flags.push((flag, Some(after))),
                        Forbidden  => return Err(ParseError::ForbiddenValue { flag })
                    }
                }
                else {
                    return Err(ParseError::UnknownArgument { attempt: before })
                }
            }
            else {
                for byte in &bytes[1..] {
                    // TODO: gotta check that these don't take arguments
                    // like -c4
                    if let Some(&Arg { short, long, takes_value }) = args.lookup_short(*byte) {
                        let flag = Flag::Short(*byte);
                        match takes_value {
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
                    else {
                        return Err(ParseError::UnknownShortArgument { attempt: *byte });
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
    use std::ffi::OsString;

    static TEST_ARGS: &'static [Arg] = &[
        Arg { short: Some(b'l'), long: "long",  takes_value: TakesValue::Forbidden },
        Arg { short: Some(b'c'), long: "count", takes_value: TakesValue::Necessary }
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

}
