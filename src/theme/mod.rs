use ansi_term::Style;

use crate::fs::File;
use crate::output::file_name::Colours as FileNameColours;
use crate::output::render;

mod ui_styles;
pub use self::ui_styles::UiStyles;
pub use self::ui_styles::Size as SizeColours;

mod lsc;
pub use self::lsc::LSColors;

mod default_theme;


#[derive(PartialEq, Eq, Debug)]
pub struct Options {

    pub use_colours: UseColours,

    pub colour_scale: ColourScale,

    pub definitions: Definitions,
}

/// Under what circumstances we should display coloured, rather than plain,
/// output to the terminal.
///
/// By default, we want to display the colours when stdout can display them.
/// Turning them on when output is going to, say, a pipe, would make programs
/// such as `grep` or `more` not work properly. So the `Automatic` mode does
/// this check and only displays colours when they can be truly appreciated.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum UseColours {

    /// Display them even when output isn’t going to a terminal.
    Always,

    /// Display them when output is going to a terminal, but not otherwise.
    Automatic,

    /// Never display them, even when output is going to a terminal.
    Never,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum ColourScale {
    Fixed,
    Gradient,
}

#[derive(PartialEq, Eq, Debug, Default)]
pub struct Definitions {
    pub ls: Option<String>,
    pub exa: Option<String>,
}


pub struct Theme {
    pub ui: UiStyles,
    pub exts: Box<dyn FileColours>,
}

impl Options {

    #[allow(trivial_casts)]   // the `as Box<_>` stuff below warns about this for some reason
    pub fn to_theme(&self, isatty: bool) -> Theme {
        use crate::info::filetype::FileExtensions;

        if self.use_colours == UseColours::Never || (self.use_colours == UseColours::Automatic && ! isatty) {
            let ui = UiStyles::plain();
            let exts = Box::new(NoFileColours);
            return Theme { ui, exts };
        }

        // Parse the environment variables into colours and extension mappings
        let mut ui = UiStyles::default_theme(self.colour_scale);
        let (exts, use_default_filetypes) = self.definitions.parse_color_vars(&mut ui);

        // Use between 0 and 2 file name highlighters
        let exts = match (exts.is_non_empty(), use_default_filetypes) {
            (false, false)  => Box::new(NoFileColours)           as Box<_>,
            (false,  true)  => Box::new(FileExtensions)          as Box<_>,
            ( true, false)  => Box::new(exts)                    as Box<_>,
            ( true,  true)  => Box::new((exts, FileExtensions))  as Box<_>,
        };

        Theme { ui, exts }
    }
}

impl Definitions {

    /// Parse the environment variables into `LS_COLORS` pairs, putting file glob
    /// colours into the `ExtensionMappings` that gets returned, and using the
    /// two-character UI codes to modify the mutable `Colours`.
    ///
    /// Also returns if the `EXA_COLORS` variable should reset the existing file
    /// type mappings or not. The `reset` code needs to be the first one.
    fn parse_color_vars(&self, colours: &mut UiStyles) -> (ExtensionMappings, bool) {
        use log::*;

        let mut exts = ExtensionMappings::default();

        if let Some(lsc) = &self.ls {
            LSColors(lsc).each_pair(|pair| {
                if ! colours.set_ls(&pair) {
                    match glob::Pattern::new(pair.key) {
                        Ok(pat) => {
                            exts.add(pat, pair.to_style());
                        }
                        Err(e) => {
                            warn!("Couldn't parse glob pattern {:?}: {}", pair.key, e);
                        }
                    }
                }
            });
        }

        let mut use_default_filetypes = true;

        if let Some(exa) = &self.exa {
            // Is this hacky? Yes.
            if exa == "reset" || exa.starts_with("reset:") {
                use_default_filetypes = false;
            }

            LSColors(exa).each_pair(|pair| {
                if ! colours.set_ls(&pair) && ! colours.set_exa(&pair) {
                    match glob::Pattern::new(pair.key) {
                        Ok(pat) => {
                            exts.add(pat, pair.to_style());
                        }
                        Err(e) => {
                            warn!("Couldn't parse glob pattern {:?}: {}", pair.key, e);
                        }
                    }
                };
            });
        }

        (exts, use_default_filetypes)
    }
}


pub trait FileColours: std::marker::Sync {
    fn colour_file(&self, file: &File<'_>) -> Option<Style>;
}

#[derive(PartialEq, Debug)]
struct NoFileColours;
impl FileColours for NoFileColours {
    fn colour_file(&self, _file: &File<'_>) -> Option<Style> {
        None
    }
}

// When getting the colour of a file from a *pair* of colourisers, try the
// first one then try the second one. This lets the user provide their own
// file type associations, while falling back to the default set if not set
// explicitly.
impl<A, B> FileColours for (A, B)
where A: FileColours,
      B: FileColours,
{
    fn colour_file(&self, file: &File<'_>) -> Option<Style> {
        self.0.colour_file(file)
            .or_else(|| self.1.colour_file(file))
    }
}


#[derive(PartialEq, Debug, Default)]
struct ExtensionMappings {
    mappings: Vec<(glob::Pattern, Style)>,
}

// Loop through backwards so that colours specified later in the list override
// colours specified earlier, like we do with options and strict mode

impl FileColours for ExtensionMappings {
    fn colour_file(&self, file: &File<'_>) -> Option<Style> {
        self.mappings.iter().rev()
            .find(|t| t.0.matches(&file.name))
            .map (|t| t.1)
    }
}

impl ExtensionMappings {
    fn is_non_empty(&self) -> bool {
        ! self.mappings.is_empty()
    }

    fn add(&mut self, pattern: glob::Pattern, style: Style) {
        self.mappings.push((pattern, style))
    }
}




impl render::BlocksColours for Theme {
    fn block_count(&self)  -> Style { self.ui.blocks }
    fn no_blocks(&self)    -> Style { self.ui.punctuation }
}

impl render::FiletypeColours for Theme {
    fn normal(&self)       -> Style { self.ui.filekinds.normal }
    fn directory(&self)    -> Style { self.ui.filekinds.directory }
    fn pipe(&self)         -> Style { self.ui.filekinds.pipe }
    fn symlink(&self)      -> Style { self.ui.filekinds.symlink }
    fn block_device(&self) -> Style { self.ui.filekinds.block_device }
    fn char_device(&self)  -> Style { self.ui.filekinds.char_device }
    fn socket(&self)       -> Style { self.ui.filekinds.socket }
    fn special(&self)      -> Style { self.ui.filekinds.special }
}

impl render::GitColours for Theme {
    fn not_modified(&self)  -> Style { self.ui.punctuation }
    #[allow(clippy::new_ret_no_self)]
    fn new(&self)           -> Style { self.ui.git.new }
    fn modified(&self)      -> Style { self.ui.git.modified }
    fn deleted(&self)       -> Style { self.ui.git.deleted }
    fn renamed(&self)       -> Style { self.ui.git.renamed }
    fn type_change(&self)   -> Style { self.ui.git.typechange }
    fn ignored(&self)       -> Style { self.ui.git.ignored }
    fn conflicted(&self)    -> Style { self.ui.git.conflicted }
}

#[cfg(unix)]
impl render::GroupColours for Theme {
    fn yours(&self)      -> Style { self.ui.users.group_yours }
    fn not_yours(&self)  -> Style { self.ui.users.group_not_yours }
}

impl render::LinksColours for Theme {
    fn normal(&self)           -> Style { self.ui.links.normal }
    fn multi_link_file(&self)  -> Style { self.ui.links.multi_link_file }
}

impl render::PermissionsColours for Theme {
    fn dash(&self)               -> Style { self.ui.punctuation }
    fn user_read(&self)          -> Style { self.ui.perms.user_read }
    fn user_write(&self)         -> Style { self.ui.perms.user_write }
    fn user_execute_file(&self)  -> Style { self.ui.perms.user_execute_file }
    fn user_execute_other(&self) -> Style { self.ui.perms.user_execute_other }
    fn group_read(&self)         -> Style { self.ui.perms.group_read }
    fn group_write(&self)        -> Style { self.ui.perms.group_write }
    fn group_execute(&self)      -> Style { self.ui.perms.group_execute }
    fn other_read(&self)         -> Style { self.ui.perms.other_read }
    fn other_write(&self)        -> Style { self.ui.perms.other_write }
    fn other_execute(&self)      -> Style { self.ui.perms.other_execute }
    fn special_user_file(&self)  -> Style { self.ui.perms.special_user_file }
    fn special_other(&self)      -> Style { self.ui.perms.special_other }
    fn attribute(&self)          -> Style { self.ui.perms.attribute }
}

impl render::SizeColours for Theme {
    fn size(&self, prefix: Option<number_prefix::Prefix>) -> Style {
        use number_prefix::Prefix::*;

        match prefix {
            Some(Kilo | Kibi) => self.ui.size.number_kilo,
            Some(Mega | Mebi) => self.ui.size.number_mega,
            Some(Giga | Gibi) => self.ui.size.number_giga,
            Some(_)           => self.ui.size.number_huge,
            None              => self.ui.size.number_byte,
        }
    }

    fn unit(&self, prefix: Option<number_prefix::Prefix>) -> Style {
        use number_prefix::Prefix::*;

        match prefix {
            Some(Kilo | Kibi) => self.ui.size.unit_kilo,
            Some(Mega | Mebi) => self.ui.size.unit_mega,
            Some(Giga | Gibi) => self.ui.size.unit_giga,
            Some(_)           => self.ui.size.unit_huge,
            None              => self.ui.size.unit_byte,
        }
    }

    fn no_size(&self) -> Style { self.ui.punctuation }
    fn major(&self)   -> Style { self.ui.size.major }
    fn comma(&self)   -> Style { self.ui.punctuation }
    fn minor(&self)   -> Style { self.ui.size.minor }
}

#[cfg(unix)]
impl render::UserColours for Theme {
    fn you(&self)           -> Style { self.ui.users.user_you }
    fn someone_else(&self)  -> Style { self.ui.users.user_someone_else }
}

impl FileNameColours for Theme {
    fn normal_arrow(&self)        -> Style { self.ui.punctuation }
    fn broken_symlink(&self)      -> Style { self.ui.broken_symlink }
    fn broken_filename(&self)     -> Style { apply_overlay(self.ui.broken_symlink, self.ui.broken_path_overlay) }
    fn broken_control_char(&self) -> Style { apply_overlay(self.ui.control_char,   self.ui.broken_path_overlay) }
    fn control_char(&self)        -> Style { self.ui.control_char }
    fn symlink_path(&self)        -> Style { self.ui.symlink_path }
    fn executable_file(&self)     -> Style { self.ui.filekinds.executable }

    fn colour_file(&self, file: &File<'_>) -> Style {
        self.exts.colour_file(file).unwrap_or(self.ui.filekinds.normal)
    }
}


/// Some of the styles are **overlays**: although they have the same attribute
/// set as regular styles (foreground and background colours, bold, underline,
/// etc), they’re intended to be used to *amend* existing styles.
///
/// For example, the target path of a broken symlink is displayed in a red,
/// underlined style by default. Paths can contain control characters, so
/// these control characters need to be underlined too, otherwise it looks
/// weird. So instead of having four separate configurable styles for “link
/// path”, “broken link path”, “control character” and “broken control
/// character”, there are styles for “link path”, “control character”, and
/// “broken link overlay”, the latter of which is just set to override the
/// underline attribute on the other two.
fn apply_overlay(mut base: Style, overlay: Style) -> Style {
    if let Some(fg) = overlay.foreground { base.foreground = Some(fg); }
    if let Some(bg) = overlay.background { base.background = Some(bg); }

    if overlay.is_bold          { base.is_bold          = true; }
    if overlay.is_dimmed        { base.is_dimmed        = true; }
    if overlay.is_italic        { base.is_italic        = true; }
    if overlay.is_underline     { base.is_underline     = true; }
    if overlay.is_blink         { base.is_blink         = true; }
    if overlay.is_reverse       { base.is_reverse       = true; }
    if overlay.is_hidden        { base.is_hidden        = true; }
    if overlay.is_strikethrough { base.is_strikethrough = true; }

    base
}
// TODO: move this function to the ansi_term crate


#[cfg(test)]
mod customs_test {
    use super::*;
    use crate::theme::ui_styles::UiStyles;
    use ansi_term::Colour::*;

    macro_rules! test {
        ($name:ident:  ls $ls:expr, exa $exa:expr  =>  colours $expected:ident -> $process_expected:expr) => {
            #[test]
            fn $name() {
                let mut $expected = UiStyles::default();
                $process_expected();

                let definitions = Definitions {
                    ls:  Some($ls.into()),
                    exa: Some($exa.into()),
                };

                let mut result = UiStyles::default();
                let (_exts, _reset) = definitions.parse_color_vars(&mut result);
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

                let definitions = Definitions {
                    ls:  Some($ls.into()),
                    exa: Some($exa.into()),
                };

                let (result, _reset) = definitions.parse_color_vars(&mut UiStyles::default());
                assert_eq!(ExtensionMappings { mappings }, result);
            }
        };
        ($name:ident:  ls $ls:expr, exa $exa:expr  =>  colours $expected:ident -> $process_expected:expr, exts $mappings:expr) => {
            #[test]
            fn $name() {
                let mut $expected = UiStyles::colourful(false);
                $process_expected();

                let mappings: Vec<(glob::Pattern, Style)>
                    = $mappings.into_iter()
                               .map(|t| (glob::Pattern::new(t.0).unwrap(), t.1))
                               .collect();

                let definitions = Definitions {
                    ls:  Some($ls.into()),
                    exa: Some($exa.into()),
                };

                let mut meh = UiStyles::colourful(false);
                let (result, _reset) = definitions.parse_color_vars(&vars, &mut meh);
                assert_eq!(ExtensionMappings { mappings }, result);
                assert_eq!($expected, meh);
            }
        };
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
