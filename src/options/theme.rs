use crate::options::{flags, vars, Vars, OptionsError};
use crate::options::parser::MatchedFlags;
use crate::theme::{Options, UseColours, ColourScale, Definitions};


impl Options {
    pub fn deduce<V: Vars>(matches: &MatchedFlags<'_>, vars: &V) -> Result<Self, OptionsError> {
        let use_colours = UseColours::deduce(matches, vars)?;
        let colour_scale = ColourScale::deduce(matches)?;

        let definitions = if use_colours == UseColours::Never {
                Definitions::default()
            }
            else {
                Definitions::deduce(vars)
            };

        Ok(Self { use_colours, colour_scale, definitions })
    }
}


impl UseColours {
    fn deduce<V: Vars>(matches: &MatchedFlags<'_>, vars: &V) -> Result<Self, OptionsError> {
        let default_value = match vars.get(vars::NO_COLOR) {
            Some(_) => Self::Never,
            None => Self::Automatic,
        };

        let word = match matches.get_where(|f| f.matches(&flags::COLOR) || f.matches(&flags::COLOUR))? {
            Some(w)  => w,
            None => return Ok(default_value),
        };

        if word == "always" {
            Ok(Self::Always)
        }
        else if word == "auto" || word == "automatic" {
            Ok(Self::Automatic)
        }
        else if word == "never" {
            Ok(Self::Never)
        }
        else {
            Err(OptionsError::BadArgument(&flags::COLOR, word.into()))
        }
    }
}


impl ColourScale {
    fn deduce(matches: &MatchedFlags<'_>) -> Result<Self, OptionsError> {
        if matches.has_where(|f| f.matches(&flags::COLOR_SCALE) || f.matches(&flags::COLOUR_SCALE))?.is_some() {
            Ok(Self::Gradient)
        }
        else {
            Ok(Self::Fixed)
        }
    }
}


impl Definitions {
    fn deduce<V: Vars>(vars: &V) -> Self {
        let ls =  vars.get(vars::LS_COLORS) .map(|e| e.to_string_lossy().to_string());
        let exa = vars.get(vars::EXA_COLORS).map(|e| e.to_string_lossy().to_string());
        Self { ls, exa }
    }
}


#[cfg(test)]
mod terminal_test {
    use super::*;
    use std::ffi::OsString;
    use crate::options::flags;
    use crate::options::parser::{Flag, Arg};

    use crate::options::test::parse_for_test;
    use crate::options::test::Strictnesses::*;

    static TEST_ARGS: &[&Arg] = &[ &flags::COLOR,       &flags::COLOUR,
                                   &flags::COLOR_SCALE, &flags::COLOUR_SCALE, ];

    macro_rules! test {
        ($name:ident:  $type:ident <- $inputs:expr;  $stricts:expr => $result:expr) => {
            #[test]
            fn $name() {
                for result in parse_for_test($inputs.as_ref(), TEST_ARGS, $stricts, |mf| $type::deduce(mf)) {
                    assert_eq!(result, $result);
                }
            }
        };

        ($name:ident:  $type:ident <- $inputs:expr, $env:expr;  $stricts:expr => $result:expr) => {
            #[test]
            fn $name() {
                let env = $env;
                for result in parse_for_test($inputs.as_ref(), TEST_ARGS, $stricts, |mf| $type::deduce(mf, &env)) {
                    assert_eq!(result, $result);
                }
            }
        };

        ($name:ident:  $type:ident <- $inputs:expr;  $stricts:expr => err $result:expr) => {
            #[test]
            fn $name() {
                for result in parse_for_test($inputs.as_ref(), TEST_ARGS, $stricts, |mf| $type::deduce(mf)) {
                    assert_eq!(result.unwrap_err(), $result);
                }
            }
        };

        ($name:ident:  $type:ident <- $inputs:expr, $env:expr;  $stricts:expr => err $result:expr) => {
            #[test]
            fn $name() {
                let env = $env;
                for result in parse_for_test($inputs.as_ref(), TEST_ARGS, $stricts, |mf| $type::deduce(mf, &env)) {
                    assert_eq!(result.unwrap_err(), $result);
                }
            }
        };
    }

    struct MockVars {
        ls: &'static str,
        exa: &'static str,
        no_color: &'static str,
    }

    impl MockVars {
        fn empty() -> MockVars {
            MockVars {
                ls: "",
                exa: "",
                no_color: "",
            }
        }
        fn with_no_color() -> MockVars {
            MockVars {
                ls: "",
                exa: "",
                no_color: "true",
            }
        }
    }

    // Test impl that just returns the value it has.
    impl Vars for MockVars {
        fn get(&self, name: &'static str) -> Option<OsString> {
            if name == vars::LS_COLORS && ! self.ls.is_empty() {
                Some(OsString::from(self.ls.clone()))
            }
            else if name == vars::EXA_COLORS && ! self.exa.is_empty() {
                Some(OsString::from(self.exa.clone()))
            }
            else if name == vars::NO_COLOR && ! self.no_color.is_empty() {
                Some(OsString::from(self.no_color.clone()))
            }
            else {
                None
            }
        }
    }



    // Default
    test!(empty:         UseColours <- [], MockVars::empty();                     Both => Ok(UseColours::Automatic));
    test!(empty_with_no_color: UseColours <- [], MockVars::with_no_color();             Both => Ok(UseColours::Never));

    // --colour
    test!(u_always:      UseColours <- ["--colour=always"], MockVars::empty();    Both => Ok(UseColours::Always));
    test!(u_auto:        UseColours <- ["--colour", "auto"], MockVars::empty();   Both => Ok(UseColours::Automatic));
    test!(u_never:       UseColours <- ["--colour=never"], MockVars::empty();     Both => Ok(UseColours::Never));

    // --color
    test!(no_u_always:   UseColours <- ["--color", "always"], MockVars::empty();  Both => Ok(UseColours::Always));
    test!(no_u_auto:     UseColours <- ["--color=auto"], MockVars::empty();       Both => Ok(UseColours::Automatic));
    test!(no_u_never:    UseColours <- ["--color", "never"], MockVars::empty();   Both => Ok(UseColours::Never));

    // Errors
    test!(no_u_error:    UseColours <- ["--color=upstream"], MockVars::empty();   Both => err OptionsError::BadArgument(&flags::COLOR, OsString::from("upstream"))); // the error is for --color
    test!(u_error:       UseColours <- ["--colour=lovers"], MockVars::empty();    Both => err OptionsError::BadArgument(&flags::COLOR, OsString::from("lovers"))); // and so is this one!

    // Overriding
    test!(overridden_1:  UseColours <- ["--colour=auto", "--colour=never"], MockVars::empty();  Last => Ok(UseColours::Never));
    test!(overridden_2:  UseColours <- ["--color=auto",  "--colour=never"], MockVars::empty();  Last => Ok(UseColours::Never));
    test!(overridden_3:  UseColours <- ["--colour=auto", "--color=never"], MockVars::empty();   Last => Ok(UseColours::Never));
    test!(overridden_4:  UseColours <- ["--color=auto",  "--color=never"], MockVars::empty();   Last => Ok(UseColours::Never));

    test!(overridden_5:  UseColours <- ["--colour=auto", "--colour=never"], MockVars::empty();  Complain => err OptionsError::Duplicate(Flag::Long("colour"), Flag::Long("colour")));
    test!(overridden_6:  UseColours <- ["--color=auto",  "--colour=never"], MockVars::empty();  Complain => err OptionsError::Duplicate(Flag::Long("color"),  Flag::Long("colour")));
    test!(overridden_7:  UseColours <- ["--colour=auto", "--color=never"], MockVars::empty();   Complain => err OptionsError::Duplicate(Flag::Long("colour"), Flag::Long("color")));
    test!(overridden_8:  UseColours <- ["--color=auto",  "--color=never"], MockVars::empty();   Complain => err OptionsError::Duplicate(Flag::Long("color"),  Flag::Long("color")));

    test!(scale_1:  ColourScale <- ["--color-scale", "--colour-scale"];   Last => Ok(ColourScale::Gradient));
    test!(scale_2:  ColourScale <- ["--color-scale",                 ];   Last => Ok(ColourScale::Gradient));
    test!(scale_3:  ColourScale <- [                 "--colour-scale"];   Last => Ok(ColourScale::Gradient));
    test!(scale_4:  ColourScale <- [                                 ];   Last => Ok(ColourScale::Fixed));

    test!(scale_5:  ColourScale <- ["--color-scale", "--colour-scale"];   Complain => err OptionsError::Duplicate(Flag::Long("color-scale"),  Flag::Long("colour-scale")));
    test!(scale_6:  ColourScale <- ["--color-scale",                 ];   Complain => Ok(ColourScale::Gradient));
    test!(scale_7:  ColourScale <- [                 "--colour-scale"];   Complain => Ok(ColourScale::Gradient));
    test!(scale_8:  ColourScale <- [                                 ];   Complain => Ok(ColourScale::Fixed));
}
