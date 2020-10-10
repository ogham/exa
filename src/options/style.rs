use ansi_term::Style;

use crate::fs::File;
use crate::options::{flags, Vars, Misfire};
use crate::options::parser::MatchedFlags;
use crate::output::file_name::{FileStyle, Classify};
use crate::style::Colours;


/// Under what circumstances we should display coloured, rather than plain,
/// output to the terminal.
///
/// By default, we want to display the colours when stdout can display them.
/// Turning them on when output is going to, say, a pipe, would make programs
/// such as `grep` or `more` not work properly. So the `Automatic` mode does
/// this check and only displays colours when they can be truly appreciated.
#[derive(PartialEq, Debug)]
enum TerminalColours {

    /// Display them even when output isn’t going to a terminal.
    Always,

    /// Display them when output is going to a terminal, but not otherwise.
    Automatic,

    /// Never display them, even when output is going to a terminal.
    Never,
}

impl Default for TerminalColours {
    fn default() -> Self {
        Self::Automatic
    }
}


impl TerminalColours {

    /// Determine which terminal colour conditions to use.
    fn deduce(matches: &MatchedFlags) -> Result<Self, Misfire> {

        let word = match matches.get_where(|f| f.matches(&flags::COLOR) || f.matches(&flags::COLOUR))? {
            Some(w) => w,
            None    => return Ok(Self::default()),
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
            Err(Misfire::BadArgument(&flags::COLOR, word.into()))
        }
    }
}


/// **Styles**, which is already an overloaded term, is a pair of view option
/// sets that happen to both be affected by `LS_COLORS` and `EXA_COLORS`.
/// Because it’s better to only iterate through that once, the two are deduced
/// together.
pub struct Styles {

    /// The colours to paint user interface elements, like the date column,
    /// and file kinds, such as directories.
    pub colours: Colours,

    /// The colours to paint the names of files that match glob patterns
    /// (and the classify option).
    pub style: FileStyle,
}

impl Styles {

    #[allow(trivial_casts)]   // the "as Box<_>" stuff below warns about this for some reason
    pub fn deduce<V, TW>(matches: &MatchedFlags, vars: &V, widther: TW) -> Result<Self, Misfire>
    where TW: Fn() -> Option<usize>, V: Vars {
        use crate::info::filetype::FileExtensions;
        use crate::output::file_name::NoFileColours;

        let classify = Classify::deduce(matches)?;

        // Before we do anything else, figure out if we need to consider
        // custom colours at all
        let tc = TerminalColours::deduce(matches)?;

        if tc == TerminalColours::Never
        || (tc == TerminalColours::Automatic && widther().is_none())
        {
            return Ok(Self {
                colours: Colours::plain(),
                style: FileStyle { classify, exts: Box::new(NoFileColours) },
            });
        }

        // Parse the environment variables into colours and extension mappings
        let scale = matches.has_where(|f| f.matches(&flags::COLOR_SCALE) || f.matches(&flags::COLOUR_SCALE))?;
        let mut colours = Colours::colourful(scale.is_some());

        let (exts, use_default_filetypes) = parse_color_vars(vars, &mut colours);

        // Use between 0 and 2 file name highlighters
        let exts = match (exts.is_non_empty(), use_default_filetypes) {
            (false, false)  => Box::new(NoFileColours)           as Box<_>,
            (false,  true)  => Box::new(FileExtensions)          as Box<_>,
            ( true, false)  => Box::new(exts)                    as Box<_>,
            ( true,  true)  => Box::new((exts, FileExtensions))  as Box<_>,
        };

        let style = FileStyle { classify, exts };
        Ok(Self { colours, style })
    }
}

/// Parse the environment variables into LS_COLORS pairs, putting file glob
/// colours into the `ExtensionMappings` that gets returned, and using the
/// two-character UI codes to modify the mutable `Colours`.
///
/// Also returns if the EXA_COLORS variable should reset the existing file
/// type mappings or not. The `reset` code needs to be the first one.
fn parse_color_vars<V: Vars>(vars: &V, colours: &mut Colours) -> (ExtensionMappings, bool) {
    use log::*;

    use crate::options::vars;
    use crate::style::LSColors;

    let mut exts = ExtensionMappings::default();

    if let Some(lsc) = vars.get(vars::LS_COLORS) {
        let lsc = lsc.to_string_lossy();
        LSColors(lsc.as_ref()).each_pair(|pair| {
            if !colours.set_ls(&pair) {
                match glob::Pattern::new(pair.key) {
                    Ok(pat) => exts.add(pat, pair.to_style()),
                    Err(e)  => warn!("Couldn't parse glob pattern {:?}: {}", pair.key, e),
                }
            }
        });
    }

    let mut use_default_filetypes = true;

    if let Some(exa) = vars.get(vars::EXA_COLORS) {
        let exa = exa.to_string_lossy();

        // Is this hacky? Yes.
        if exa == "reset" || exa.starts_with("reset:") {
            use_default_filetypes = false;
        }

        LSColors(exa.as_ref()).each_pair(|pair| {
            if !colours.set_ls(&pair) && !colours.set_exa(&pair) {
                match glob::Pattern::new(pair.key) {
                    Ok(pat) => exts.add(pat, pair.to_style()),
                    Err(e)  => warn!("Couldn't parse glob pattern {:?}: {}", pair.key, e),
                }
            };
        });
    }

    (exts, use_default_filetypes)
}


#[derive(PartialEq, Debug, Default)]
struct ExtensionMappings {
    mappings: Vec<(glob::Pattern, Style)>
}

// Loop through backwards so that colours specified later in the list override
// colours specified earlier, like we do with options and strict mode

use crate::output::file_name::FileColours;
impl FileColours for ExtensionMappings {
    fn colour_file(&self, file: &File) -> Option<Style> {
        self.mappings
            .iter()
            .rev()
            .find(|t| t.0.matches(&file.name))
            .map (|t| t.1)
    }
}

impl ExtensionMappings {
    fn is_non_empty(&self) -> bool {
        !self.mappings.is_empty()
    }

    fn add(&mut self, pattern: glob::Pattern, style: Style) {
        self.mappings.push((pattern, style))
    }
}



impl Classify {
    fn deduce(matches: &MatchedFlags) -> Result<Self, Misfire> {
        let flagged = matches.has(&flags::CLASSIFY)?;

        Ok(if flagged { Self::AddFileIndicators }
                 else { Self::JustFilenames })
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

    static TEST_ARGS: &[&Arg] = &[ &flags::COLOR, &flags::COLOUR ];

    macro_rules! test {
        ($name:ident:  $inputs:expr;  $stricts:expr => $result:expr) => {
            #[test]
            fn $name() {
                for result in parse_for_test($inputs.as_ref(), TEST_ARGS, $stricts, |mf| TerminalColours::deduce(mf)) {
                    assert_eq!(result, $result);
                }
            }
        };

        ($name:ident:  $inputs:expr;  $stricts:expr => err $result:expr) => {
            #[test]
            fn $name() {
                for result in parse_for_test($inputs.as_ref(), TEST_ARGS, $stricts, |mf| TerminalColours::deduce(mf)) {
                    assert_eq!(result.unwrap_err(), $result);
                }
            }
        };
    }


    // Default
    test!(empty:         [];                     Both => Ok(TerminalColours::default()));

    // --colour
    test!(u_always:      ["--colour=always"];    Both => Ok(TerminalColours::Always));
    test!(u_auto:        ["--colour", "auto"];   Both => Ok(TerminalColours::Automatic));
    test!(u_never:       ["--colour=never"];     Both => Ok(TerminalColours::Never));

    // --color
    test!(no_u_always:   ["--color", "always"];  Both => Ok(TerminalColours::Always));
    test!(no_u_auto:     ["--color=auto"];       Both => Ok(TerminalColours::Automatic));
    test!(no_u_never:    ["--color", "never"];   Both => Ok(TerminalColours::Never));

    // Errors
    test!(no_u_error:    ["--color=upstream"];   Both => err Misfire::BadArgument(&flags::COLOR, OsString::from("upstream")));  // the error is for --color
    test!(u_error:       ["--colour=lovers"];    Both => err Misfire::BadArgument(&flags::COLOR, OsString::from("lovers")));    // and so is this one!

    // Overriding
    test!(overridden_1:  ["--colour=auto", "--colour=never"];  Last => Ok(TerminalColours::Never));
    test!(overridden_2:  ["--color=auto",  "--colour=never"];  Last => Ok(TerminalColours::Never));
    test!(overridden_3:  ["--colour=auto", "--color=never"];   Last => Ok(TerminalColours::Never));
    test!(overridden_4:  ["--color=auto",  "--color=never"];   Last => Ok(TerminalColours::Never));

    test!(overridden_5:  ["--colour=auto", "--colour=never"];  Complain => err Misfire::Duplicate(Flag::Long("colour"), Flag::Long("colour")));
    test!(overridden_6:  ["--color=auto",  "--colour=never"];  Complain => err Misfire::Duplicate(Flag::Long("color"),  Flag::Long("colour")));
    test!(overridden_7:  ["--colour=auto", "--color=never"];   Complain => err Misfire::Duplicate(Flag::Long("colour"), Flag::Long("color")));
    test!(overridden_8:  ["--color=auto",  "--color=never"];   Complain => err Misfire::Duplicate(Flag::Long("color"),  Flag::Long("color")));
}


#[cfg(test)]
mod colour_test {
    use super::*;
    use crate::options::flags;
    use crate::options::parser::{Flag, Arg};

    use crate::options::test::parse_for_test;
    use crate::options::test::Strictnesses::*;

    static TEST_ARGS: &[&Arg] = &[ &flags::COLOR,       &flags::COLOUR,
                                   &flags::COLOR_SCALE, &flags::COLOUR_SCALE ];

    macro_rules! test {
        ($name:ident:  $inputs:expr, $widther:expr;  $stricts:expr => $result:expr) => {
            #[test]
            fn $name() {
                for result in parse_for_test($inputs.as_ref(), TEST_ARGS, $stricts, |mf| Styles::deduce(mf, &None, &$widther).map(|s| s.colours)) {
                    assert_eq!(result, $result);
                }
            }
        };

        ($name:ident:  $inputs:expr, $widther:expr;  $stricts:expr => err $result:expr) => {
            #[test]
            fn $name() {
                for result in parse_for_test($inputs.as_ref(), TEST_ARGS, $stricts, |mf| Styles::deduce(mf, &None, &$widther).map(|s| s.colours)) {
                    assert_eq!(result.unwrap_err(), $result);
                }
            }
        };

        ($name:ident:  $inputs:expr, $widther:expr;  $stricts:expr => like $pat:pat) => {
            #[test]
            fn $name() {
                for result in parse_for_test($inputs.as_ref(), TEST_ARGS, $stricts, |mf| Styles::deduce(mf, &None, &$widther).map(|s| s.colours)) {
                    println!("Testing {:?}", result);
                    match result {
                        $pat => assert!(true),
                        _    => assert!(false),
                    }
                }
            }
        };
    }

    test!(width_1:  ["--colour", "always"],    || Some(80);  Both => Ok(Colours::colourful(false)));
    test!(width_2:  ["--colour", "always"],    || None;      Both => Ok(Colours::colourful(false)));
    test!(width_3:  ["--colour", "never"],     || Some(80);  Both => Ok(Colours::plain()));
    test!(width_4:  ["--colour", "never"],     || None;      Both => Ok(Colours::plain()));
    test!(width_5:  ["--colour", "automatic"], || Some(80);  Both => Ok(Colours::colourful(false)));
    test!(width_6:  ["--colour", "automatic"], || None;      Both => Ok(Colours::plain()));
    test!(width_7:  [],                        || Some(80);  Both => Ok(Colours::colourful(false)));
    test!(width_8:  [],                        || None;      Both => Ok(Colours::plain()));

    test!(scale_1:  ["--color=always", "--color-scale", "--colour-scale"], || None;   Last => Ok(Colours::colourful(true)));
    test!(scale_2:  ["--color=always", "--color-scale",                 ], || None;   Last => Ok(Colours::colourful(true)));
    test!(scale_3:  ["--color=always",                  "--colour-scale"], || None;   Last => Ok(Colours::colourful(true)));
    test!(scale_4:  ["--color=always",                                  ], || None;   Last => Ok(Colours::colourful(false)));

    test!(scale_5:  ["--color=always", "--color-scale", "--colour-scale"], || None;   Complain => err Misfire::Duplicate(Flag::Long("color-scale"),  Flag::Long("colour-scale")));
    test!(scale_6:  ["--color=always", "--color-scale",                 ], || None;   Complain => Ok(Colours::colourful(true)));
    test!(scale_7:  ["--color=always",                  "--colour-scale"], || None;   Complain => Ok(Colours::colourful(true)));
    test!(scale_8:  ["--color=always",                                  ], || None;   Complain => Ok(Colours::colourful(false)));
}



#[cfg(test)]
mod customs_test {
    use std::ffi::OsString;

    use super::*;
    use crate::options::Vars;

    use ansi_term::Colour::*;

    macro_rules! test {
        ($name:ident:  ls $ls:expr, exa $exa:expr  =>  colours $expected:ident -> $process_expected:expr) => {
            #[test]
            #[allow(unused_mut)]
            fn $name() {
                let mut $expected = Colours::colourful(false);
                $process_expected();

                let vars = MockVars { ls: $ls, exa: $exa };

                let mut result = Colours::colourful(false);
                let (_exts, _reset) = parse_color_vars(&vars, &mut result);
                assert_eq!($expected, result);
            }
        };
        ($name:ident:  ls $ls:expr, exa $exa:expr  =>  exts $mappings:expr) => {
            #[test]
            fn $name() {
                let mappings: Vec<(glob::Pattern, Style)>
                    = $mappings.iter()
                               .map(|t| (glob::Pattern::new(t.0).unwrap(), t.1))
                               .collect();

                let vars = MockVars { ls: $ls, exa: $exa };

                let mut meh = Colours::colourful(false);
                let (result, _reset) = parse_color_vars(&vars, &mut meh);
                assert_eq!(ExtensionMappings { mappings }, result);
            }
        };
        ($name:ident:  ls $ls:expr, exa $exa:expr  =>  colours $expected:ident -> $process_expected:expr, exts $mappings:expr) => {
            #[test]
            #[allow(unused_mut)]
            fn $name() {
                let mut $expected = Colours::colourful(false);
                $process_expected();

                let mappings: Vec<(glob::Pattern, Style)>
                    = $mappings.into_iter()
                               .map(|t| (glob::Pattern::new(t.0).unwrap(), t.1))
                               .collect();

                let vars = MockVars { ls: $ls, exa: $exa };

                let mut meh = Colours::colourful(false);
                let (result, _reset) = parse_color_vars(&vars, &mut meh);
                assert_eq!(ExtensionMappings { mappings }, result);
                assert_eq!($expected, meh);
            }
        };
    }

    struct MockVars {
        ls: &'static str,
        exa: &'static str,
    }

    // Test impl that just returns the value it has.
    impl Vars for MockVars {
        fn get(&self, name: &'static str) -> Option<OsString> {
            use crate::options::vars;

            if name == vars::LS_COLORS && !self.ls.is_empty() {
                OsString::from(self.ls.clone()).into()
            }
            else if name == vars::EXA_COLORS && !self.exa.is_empty() {
                OsString::from(self.exa.clone()).into()
            }
            else {
                None
            }
        }
    }

    // LS_COLORS can affect all of these colours:
    test!(ls_di:   ls "di=31", exa ""  =>  colours c -> { c.filekinds.directory    = Red.normal();    });
    test!(ls_ex:   ls "ex=32", exa ""  =>  colours c -> { c.filekinds.executable   = Green.normal();  });
    test!(ls_fi:   ls "fi=33", exa ""  =>  colours c -> { c.filekinds.normal       = Yellow.normal(); });
    test!(ls_pi:   ls "pi=34", exa ""  =>  colours c -> { c.filekinds.pipe         = Blue.normal();   });
    test!(ls_so:   ls "so=35", exa ""  =>  colours c -> { c.filekinds.socket       = Purple.normal(); });
    test!(ls_bd:   ls "bd=36", exa ""  =>  colours c -> { c.filekinds.block_device = Cyan.normal();   });
    test!(ls_cd:   ls "cd=35", exa ""  =>  colours c -> { c.filekinds.char_device  = Purple.normal(); });
    test!(ls_ln:   ls "ln=34", exa ""  =>  colours c -> { c.filekinds.symlink      = Blue.normal();   });
    test!(ls_or:   ls "or=33", exa ""  =>  colours c -> { c.broken_symlink         = Yellow.normal(); });

    // EXA_COLORS can affect all those colours too:
    test!(exa_di:  ls "", exa "di=32"  =>  colours c -> { c.filekinds.directory    = Green.normal();  });
    test!(exa_ex:  ls "", exa "ex=33"  =>  colours c -> { c.filekinds.executable   = Yellow.normal(); });
    test!(exa_fi:  ls "", exa "fi=34"  =>  colours c -> { c.filekinds.normal       = Blue.normal();   });
    test!(exa_pi:  ls "", exa "pi=35"  =>  colours c -> { c.filekinds.pipe         = Purple.normal(); });
    test!(exa_so:  ls "", exa "so=36"  =>  colours c -> { c.filekinds.socket       = Cyan.normal();   });
    test!(exa_bd:  ls "", exa "bd=35"  =>  colours c -> { c.filekinds.block_device = Purple.normal(); });
    test!(exa_cd:  ls "", exa "cd=34"  =>  colours c -> { c.filekinds.char_device  = Blue.normal();   });
    test!(exa_ln:  ls "", exa "ln=33"  =>  colours c -> { c.filekinds.symlink      = Yellow.normal(); });
    test!(exa_or:  ls "", exa "or=32"  =>  colours c -> { c.broken_symlink         = Green.normal();  });

    // EXA_COLORS will even override options from LS_COLORS:
    test!(ls_exa_di: ls "di=31", exa "di=32"  =>  colours c -> { c.filekinds.directory  = Green.normal();  });
    test!(ls_exa_ex: ls "ex=32", exa "ex=33"  =>  colours c -> { c.filekinds.executable = Yellow.normal(); });
    test!(ls_exa_fi: ls "fi=33", exa "fi=34"  =>  colours c -> { c.filekinds.normal     = Blue.normal();   });

    // But more importantly, EXA_COLORS has its own, special list of colours:
    test!(exa_ur:  ls "", exa "ur=38;5;100"  =>  colours c -> { c.perms.user_read           = Fixed(100).normal(); });
    test!(exa_uw:  ls "", exa "uw=38;5;101"  =>  colours c -> { c.perms.user_write          = Fixed(101).normal(); });
    test!(exa_ux:  ls "", exa "ux=38;5;102"  =>  colours c -> { c.perms.user_execute_file   = Fixed(102).normal(); });
    test!(exa_ue:  ls "", exa "ue=38;5;103"  =>  colours c -> { c.perms.user_execute_other  = Fixed(103).normal(); });
    test!(exa_gr:  ls "", exa "gr=38;5;104"  =>  colours c -> { c.perms.group_read          = Fixed(104).normal(); });
    test!(exa_gw:  ls "", exa "gw=38;5;105"  =>  colours c -> { c.perms.group_write         = Fixed(105).normal(); });
    test!(exa_gx:  ls "", exa "gx=38;5;106"  =>  colours c -> { c.perms.group_execute       = Fixed(106).normal(); });
    test!(exa_tr:  ls "", exa "tr=38;5;107"  =>  colours c -> { c.perms.other_read          = Fixed(107).normal(); });
    test!(exa_tw:  ls "", exa "tw=38;5;108"  =>  colours c -> { c.perms.other_write         = Fixed(108).normal(); });
    test!(exa_tx:  ls "", exa "tx=38;5;109"  =>  colours c -> { c.perms.other_execute       = Fixed(109).normal(); });
    test!(exa_su:  ls "", exa "su=38;5;110"  =>  colours c -> { c.perms.special_user_file   = Fixed(110).normal(); });
    test!(exa_sf:  ls "", exa "sf=38;5;111"  =>  colours c -> { c.perms.special_other       = Fixed(111).normal(); });
    test!(exa_xa:  ls "", exa "xa=38;5;112"  =>  colours c -> { c.perms.attribute           = Fixed(112).normal(); });

    test!(exa_sn:  ls "", exa "sn=38;5;113" => colours c -> {
        c.size.number_byte = Fixed(113).normal();
        c.size.number_kilo = Fixed(113).normal();
        c.size.number_mega = Fixed(113).normal();
        c.size.number_giga = Fixed(113).normal();
        c.size.number_huge = Fixed(113).normal();
    });
    test!(exa_sb:  ls "", exa "sb=38;5;114" => colours c -> {
        c.size.unit_byte = Fixed(114).normal();
        c.size.unit_kilo = Fixed(114).normal();
        c.size.unit_mega = Fixed(114).normal();
        c.size.unit_giga = Fixed(114).normal();
        c.size.unit_huge = Fixed(114).normal();
    });

    test!(exa_nb:  ls "", exa "nb=38;5;115"  =>  colours c -> { c.size.number_byte          = Fixed(115).normal(); });
    test!(exa_nk:  ls "", exa "nk=38;5;116"  =>  colours c -> { c.size.number_kilo          = Fixed(116).normal(); });
    test!(exa_nm:  ls "", exa "nm=38;5;117"  =>  colours c -> { c.size.number_mega          = Fixed(117).normal(); });
    test!(exa_ng:  ls "", exa "ng=38;5;118"  =>  colours c -> { c.size.number_giga          = Fixed(118).normal(); });
    test!(exa_nh:  ls "", exa "nh=38;5;119"  =>  colours c -> { c.size.number_huge          = Fixed(119).normal(); });

    test!(exa_ub:  ls "", exa "ub=38;5;115"  =>  colours c -> { c.size.unit_byte            = Fixed(115).normal(); });
    test!(exa_uk:  ls "", exa "uk=38;5;116"  =>  colours c -> { c.size.unit_kilo            = Fixed(116).normal(); });
    test!(exa_um:  ls "", exa "um=38;5;117"  =>  colours c -> { c.size.unit_mega            = Fixed(117).normal(); });
    test!(exa_ug:  ls "", exa "ug=38;5;118"  =>  colours c -> { c.size.unit_giga            = Fixed(118).normal(); });
    test!(exa_uh:  ls "", exa "uh=38;5;119"  =>  colours c -> { c.size.unit_huge            = Fixed(119).normal(); });

    test!(exa_df:  ls "", exa "df=38;5;115"  =>  colours c -> { c.size.major                = Fixed(115).normal(); });
    test!(exa_ds:  ls "", exa "ds=38;5;116"  =>  colours c -> { c.size.minor                = Fixed(116).normal(); });

    test!(exa_uu:  ls "", exa "uu=38;5;117"  =>  colours c -> { c.users.user_you            = Fixed(117).normal(); });
    test!(exa_un:  ls "", exa "un=38;5;118"  =>  colours c -> { c.users.user_someone_else   = Fixed(118).normal(); });
    test!(exa_gu:  ls "", exa "gu=38;5;119"  =>  colours c -> { c.users.group_yours         = Fixed(119).normal(); });
    test!(exa_gn:  ls "", exa "gn=38;5;120"  =>  colours c -> { c.users.group_not_yours     = Fixed(120).normal(); });

    test!(exa_lc:  ls "", exa "lc=38;5;121"  =>  colours c -> { c.links.normal              = Fixed(121).normal(); });
    test!(exa_lm:  ls "", exa "lm=38;5;122"  =>  colours c -> { c.links.multi_link_file     = Fixed(122).normal(); });

    test!(exa_ga:  ls "", exa "ga=38;5;123"  =>  colours c -> { c.git.new                   = Fixed(123).normal(); });
    test!(exa_gm:  ls "", exa "gm=38;5;124"  =>  colours c -> { c.git.modified              = Fixed(124).normal(); });
    test!(exa_gd:  ls "", exa "gd=38;5;125"  =>  colours c -> { c.git.deleted               = Fixed(125).normal(); });
    test!(exa_gv:  ls "", exa "gv=38;5;126"  =>  colours c -> { c.git.renamed               = Fixed(126).normal(); });
    test!(exa_gt:  ls "", exa "gt=38;5;127"  =>  colours c -> { c.git.typechange            = Fixed(127).normal(); });

    test!(exa_xx:  ls "", exa "xx=38;5;128"  =>  colours c -> { c.punctuation               = Fixed(128).normal(); });
    test!(exa_da:  ls "", exa "da=38;5;129"  =>  colours c -> { c.date                      = Fixed(129).normal(); });
    test!(exa_in:  ls "", exa "in=38;5;130"  =>  colours c -> { c.inode                     = Fixed(130).normal(); });
    test!(exa_bl:  ls "", exa "bl=38;5;131"  =>  colours c -> { c.blocks                    = Fixed(131).normal(); });
    test!(exa_hd:  ls "", exa "hd=38;5;132"  =>  colours c -> { c.header                    = Fixed(132).normal(); });
    test!(exa_lp:  ls "", exa "lp=38;5;133"  =>  colours c -> { c.symlink_path              = Fixed(133).normal(); });
    test!(exa_cc:  ls "", exa "cc=38;5;134"  =>  colours c -> { c.control_char              = Fixed(134).normal(); });
    test!(exa_bo:  ls "", exa "bO=4"         =>  colours c -> { c.broken_path_overlay       = Style::default().underline(); });

    // All the while, LS_COLORS treats them as filenames:
    test!(ls_uu:   ls "uu=38;5;117", exa ""  =>  exts [ ("uu", Fixed(117).normal()) ]);
    test!(ls_un:   ls "un=38;5;118", exa ""  =>  exts [ ("un", Fixed(118).normal()) ]);
    test!(ls_gu:   ls "gu=38;5;119", exa ""  =>  exts [ ("gu", Fixed(119).normal()) ]);
    test!(ls_gn:   ls "gn=38;5;120", exa ""  =>  exts [ ("gn", Fixed(120).normal()) ]);

    // Just like all other keys:
    test!(ls_txt:  ls "*.txt=31",          exa ""  =>  exts [ ("*.txt",      Red.normal())             ]);
    test!(ls_mp3:  ls "*.mp3=38;5;135",    exa ""  =>  exts [ ("*.mp3",      Fixed(135).normal())      ]);
    test!(ls_mak:  ls "Makefile=1;32;4",   exa ""  =>  exts [ ("Makefile",   Green.bold().underline()) ]);
    test!(exa_txt: ls "", exa "*.zip=31"           =>  exts [ ("*.zip",      Red.normal())             ]);
    test!(exa_mp3: ls "", exa "lev.*=38;5;153"     =>  exts [ ("lev.*",      Fixed(153).normal())      ]);
    test!(exa_mak: ls "", exa "Cargo.toml=4;32;1"  =>  exts [ ("Cargo.toml", Green.bold().underline()) ]);

    // Testing whether a glob from EXA_COLORS overrides a glob from LS_COLORS
    // can’t be tested here, because they’ll both be added to the same vec

    // Values get separated by colons:
    test!(ls_multi:   ls "*.txt=31:*.rtf=32", exa ""  =>  exts [ ("*.txt", Red.normal()),   ("*.rtf", Green.normal()) ]);
    test!(exa_multi:  ls "", exa "*.tmp=37:*.log=37"  =>  exts [ ("*.tmp", White.normal()), ("*.log", White.normal()) ]);

    test!(ls_five: ls "1*1=31:2*2=32:3*3=1;33:4*4=34;1:5*5=35;4", exa ""  =>  exts [
        ("1*1", Red.normal()), ("2*2", Green.normal()), ("3*3", Yellow.bold()), ("4*4", Blue.bold()), ("5*5", Purple.underline())
    ]);

    // Finally, colours get applied right-to-left:
    test!(ls_overwrite:  ls "pi=31:pi=32:pi=33", exa ""  =>  colours c -> { c.filekinds.pipe = Yellow.normal(); });
    test!(exa_overwrite: ls "", exa "da=36:da=35:da=34"  =>  colours c -> { c.date = Blue.normal(); });
}
