use ansi_term::Style;
use ansi_term::Colour::{Red, Green, Yellow, Blue, Cyan, Purple, Fixed};

use crate::output::render;
use crate::output::file_name::Colours as FileNameColours;

use crate::style::lsc::Pair;


#[derive(Debug, Default, PartialEq)]
pub struct Colours {
    pub colourful: bool,

    pub filekinds:  FileKinds,
    pub perms:      Permissions,
    pub size:       Size,
    pub users:      Users,
    pub links:      Links,
    pub git:        Git,

    pub punctuation:  Style,
    pub date:         Style,
    pub inode:        Style,
    pub blocks:       Style,
    pub header:       Style,

    pub symlink_path:         Style,
    pub control_char:         Style,
    pub broken_symlink:       Style,
    pub broken_path_overlay:  Style,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FileKinds {
    pub normal: Style,
    pub directory: Style,
    pub symlink: Style,
    pub pipe: Style,
    pub block_device: Style,
    pub char_device: Style,
    pub socket: Style,
    pub special: Style,
    pub executable: Style,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Permissions {
    pub user_read:          Style,
    pub user_write:         Style,
    pub user_execute_file:  Style,
    pub user_execute_other: Style,

    pub group_read:    Style,
    pub group_write:   Style,
    pub group_execute: Style,

    pub other_read:    Style,
    pub other_write:   Style,
    pub other_execute: Style,

    pub special_user_file: Style,
    pub special_other:     Style,

    pub attribute: Style,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub major: Style,
    pub minor: Style,

    pub number_byte: Style,
    pub number_kilo: Style,
    pub number_mega: Style,
    pub number_giga: Style,
    pub number_huge: Style,

    pub unit_byte: Style,
    pub unit_kilo: Style,
    pub unit_mega: Style,
    pub unit_giga: Style,
    pub unit_huge: Style,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Users {
    pub user_you: Style,
    pub user_someone_else: Style,
    pub group_yours: Style,
    pub group_not_yours: Style,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Links {
    pub normal: Style,
    pub multi_link_file: Style,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Git {
    pub new: Style,
    pub modified: Style,
    pub deleted: Style,
    pub renamed: Style,
    pub typechange: Style,
    pub ignored: Style,
}

impl Colours {
    pub fn plain() -> Colours {
        Colours::default()
    }

    pub fn colourful(scale: bool) -> Colours {
        Colours {
            colourful: true,

            filekinds: FileKinds {
                normal:       Style::default(),
                directory:    Blue.bold(),
                symlink:      Cyan.normal(),
                pipe:         Yellow.normal(),
                block_device: Yellow.bold(),
                char_device:  Yellow.bold(),
                socket:       Red.bold(),
                special:      Yellow.normal(),
                executable:   Green.bold(),
            },

            perms: Permissions {
                user_read:           Yellow.bold(),
                user_write:          Red.bold(),
                user_execute_file:   Green.bold().underline(),
                user_execute_other:  Green.bold(),

                group_read:          Yellow.normal(),
                group_write:         Red.normal(),
                group_execute:       Green.normal(),

                other_read:          Yellow.normal(),
                other_write:         Red.normal(),
                other_execute:       Green.normal(),

                special_user_file:   Purple.normal(),
                special_other:       Purple.normal(),

                attribute:           Style::default(),
            },

            size: Size::colourful(scale),

            users: Users {
                user_you:           Yellow.bold(),
                user_someone_else:  Style::default(),
                group_yours:        Yellow.bold(),
                group_not_yours:    Style::default(),
            },

            links: Links {
                normal:          Red.bold(),
                multi_link_file: Red.on(Yellow),
            },

            git: Git {
                new:         Green.normal(),
                modified:    Blue.normal(),
                deleted:     Red.normal(),
                renamed:     Yellow.normal(),
                typechange:  Purple.normal(),
                ignored:     Style::default().dimmed(),
            },

            punctuation:  Fixed(244).normal(),
            date:         Blue.normal(),
            inode:        Purple.normal(),
            blocks:       Cyan.normal(),
            header:       Style::default().underline(),

            symlink_path:         Cyan.normal(),
            control_char:         Red.normal(),
            broken_symlink:       Red.normal(),
            broken_path_overlay:  Style::default().underline(),
        }
    }
}

impl Size {
    pub fn colourful(scale: bool) -> Self {
        if scale {
            Self::colourful_scale()
        } else {
            Self::colourful_plain()
        }
    }

    fn colourful_plain() -> Self {
        Self {
            major:  Green.bold(),
            minor:  Green.normal(),

            number_byte: Green.bold(),
            number_kilo: Green.bold(),
            number_mega: Green.bold(),
            number_giga: Green.bold(),
            number_huge: Green.bold(),

            unit_byte: Green.normal(),
            unit_kilo: Green.normal(),
            unit_mega: Green.normal(),
            unit_giga: Green.normal(),
            unit_huge: Green.normal(),
        }
    }

    fn colourful_scale() -> Self {
        Self {
            major:  Green.bold(),
            minor:  Green.normal(),

            number_byte: Fixed(118).normal(),
            number_kilo: Fixed(190).normal(),
            number_mega: Fixed(226).normal(),
            number_giga: Fixed(220).normal(),
            number_huge: Fixed(214).normal(),

            unit_byte: Green.normal(),
            unit_kilo: Green.normal(),
            unit_mega: Green.normal(),
            unit_giga: Green.normal(),
            unit_huge: Green.normal(),
        }

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


impl Colours {

    /// Sets a value on this set of colours using one of the keys understood
    /// by the `LS_COLORS` environment variable. Invalid keys set nothing, but
    /// return false.
    pub fn set_ls(&mut self, pair: &Pair) -> bool {
        match pair.key {
            "di" => self.filekinds.directory    = pair.to_style(),  // DIR
            "ex" => self.filekinds.executable   = pair.to_style(),  // EXEC
            "fi" => self.filekinds.normal       = pair.to_style(),  // FILE
            "pi" => self.filekinds.pipe         = pair.to_style(),  // FIFO
            "so" => self.filekinds.socket       = pair.to_style(),  // SOCK
            "bd" => self.filekinds.block_device = pair.to_style(),  // BLK
            "cd" => self.filekinds.char_device  = pair.to_style(),  // CHR
            "ln" => self.filekinds.symlink      = pair.to_style(),  // LINK
            "or" => self.broken_symlink         = pair.to_style(),  // ORPHAN
             _   => return false,
             // Codes we don’t do anything with:
             // MULTIHARDLINK, DOOR, SETUID, SETGID, CAPABILITY,
             // STICKY_OTHER_WRITABLE, OTHER_WRITABLE, STICKY, MISSING
        }
        true
    }

    /// Sets a value on this set of colours using one of the keys understood
    /// by the `EXA_COLORS` environment variable. Invalid keys set nothing,
    /// but return false. This doesn’t take the `LS_COLORS` keys into account,
    /// so `set_ls` should have been run first.
    pub fn set_exa(&mut self, pair: &Pair) -> bool {
        match pair.key {
            "ur" => self.perms.user_read          = pair.to_style(),
            "uw" => self.perms.user_write         = pair.to_style(),
            "ux" => self.perms.user_execute_file  = pair.to_style(),
            "ue" => self.perms.user_execute_other = pair.to_style(),
            "gr" => self.perms.group_read         = pair.to_style(),
            "gw" => self.perms.group_write        = pair.to_style(),
            "gx" => self.perms.group_execute      = pair.to_style(),
            "tr" => self.perms.other_read         = pair.to_style(),
            "tw" => self.perms.other_write        = pair.to_style(),
            "tx" => self.perms.other_execute      = pair.to_style(),
            "su" => self.perms.special_user_file  = pair.to_style(),
            "sf" => self.perms.special_other      = pair.to_style(),
            "xa" => self.perms.attribute          = pair.to_style(),

            "sn" => self.set_number_style(pair.to_style()),
            "sb" => self.set_unit_style(pair.to_style()),
            "nb" => self.size.number_byte         = pair.to_style(),
            "nk" => self.size.number_kilo         = pair.to_style(),
            "nm" => self.size.number_mega         = pair.to_style(),
            "ng" => self.size.number_giga         = pair.to_style(),
            "nh" => self.size.number_huge         = pair.to_style(),
            "ub" => self.size.unit_byte           = pair.to_style(),
            "uk" => self.size.unit_kilo           = pair.to_style(),
            "um" => self.size.unit_mega           = pair.to_style(),
            "ug" => self.size.unit_giga           = pair.to_style(),
            "uh" => self.size.unit_huge           = pair.to_style(),
            "df" => self.size.major               = pair.to_style(),
            "ds" => self.size.minor               = pair.to_style(),

            "uu" => self.users.user_you           = pair.to_style(),
            "un" => self.users.user_someone_else  = pair.to_style(),
            "gu" => self.users.group_yours        = pair.to_style(),
            "gn" => self.users.group_not_yours    = pair.to_style(),

            "lc" => self.links.normal             = pair.to_style(),
            "lm" => self.links.multi_link_file    = pair.to_style(),

            "ga" => self.git.new                  = pair.to_style(),
            "gm" => self.git.modified             = pair.to_style(),
            "gd" => self.git.deleted              = pair.to_style(),
            "gv" => self.git.renamed              = pair.to_style(),
            "gt" => self.git.typechange           = pair.to_style(),

            "xx" => self.punctuation              = pair.to_style(),
            "da" => self.date                     = pair.to_style(),
            "in" => self.inode                    = pair.to_style(),
            "bl" => self.blocks                   = pair.to_style(),
            "hd" => self.header                   = pair.to_style(),
            "lp" => self.symlink_path             = pair.to_style(),
            "cc" => self.control_char             = pair.to_style(),
            "bO" => self.broken_path_overlay      = pair.to_style(),

             _   => return false,
        }
        true
    }

    pub fn set_number_style(&mut self, style: Style) {
        self.size.number_byte = style;
        self.size.number_kilo = style;
        self.size.number_mega = style;
        self.size.number_giga = style;
        self.size.number_huge = style;
    }

    pub fn set_unit_style(&mut self, style: Style) {
        self.size.unit_byte = style;
        self.size.unit_kilo = style;
        self.size.unit_mega = style;
        self.size.unit_giga = style;
        self.size.unit_huge = style;
    }
}


impl render::BlocksColours for Colours {
    fn block_count(&self)  -> Style { self.blocks }
    fn no_blocks(&self)    -> Style { self.punctuation }
}

impl render::FiletypeColours for Colours {
    fn normal(&self)       -> Style { self.filekinds.normal }
    fn directory(&self)    -> Style { self.filekinds.directory }
    fn pipe(&self)         -> Style { self.filekinds.pipe }
    fn symlink(&self)      -> Style { self.filekinds.symlink }
    fn block_device(&self) -> Style { self.filekinds.block_device }
    fn char_device(&self)  -> Style { self.filekinds.char_device }
    fn socket(&self)       -> Style { self.filekinds.socket }
    fn special(&self)      -> Style { self.filekinds.special }
}

impl render::GitColours for Colours {
    fn not_modified(&self)  -> Style { self.punctuation }
    #[allow(clippy::new_ret_no_self)]
    fn new(&self)           -> Style { self.git.new }
    fn modified(&self)      -> Style { self.git.modified }
    fn deleted(&self)       -> Style { self.git.deleted }
    fn renamed(&self)       -> Style { self.git.renamed }
    fn type_change(&self)   -> Style { self.git.typechange }
    fn ignored(&self)       -> Style { self.git.ignored }
}

impl render::GroupColours for Colours {
    fn yours(&self)      -> Style { self.users.group_yours }
    fn not_yours(&self)  -> Style { self.users.group_not_yours }
}

impl render::LinksColours for Colours {
    fn normal(&self)           -> Style { self.links.normal }
    fn multi_link_file(&self)  -> Style { self.links.multi_link_file }
}

impl render::PermissionsColours for Colours {
    fn dash(&self)               -> Style { self.punctuation }
    fn user_read(&self)          -> Style { self.perms.user_read }
    fn user_write(&self)         -> Style { self.perms.user_write }
    fn user_execute_file(&self)  -> Style { self.perms.user_execute_file }
    fn user_execute_other(&self) -> Style { self.perms.user_execute_other }
    fn group_read(&self)         -> Style { self.perms.group_read }
    fn group_write(&self)        -> Style { self.perms.group_write }
    fn group_execute(&self)      -> Style { self.perms.group_execute }
    fn other_read(&self)         -> Style { self.perms.other_read }
    fn other_write(&self)        -> Style { self.perms.other_write }
    fn other_execute(&self)      -> Style { self.perms.other_execute }
    fn special_user_file(&self)  -> Style { self.perms.special_user_file }
    fn special_other(&self)      -> Style { self.perms.special_other }
    fn attribute(&self)          -> Style { self.perms.attribute }
}

impl render::SizeColours for Colours {
    fn size(&self, prefix: Option<number_prefix::Prefix>) -> Style {
        use number_prefix::Prefix::*;
        match prefix {
            None                    => self.size.number_byte,
            Some(Kilo) | Some(Kibi) => self.size.number_kilo,
            Some(Mega) | Some(Mibi) => self.size.number_mega,
            Some(Giga) | Some(Gibi) => self.size.number_giga,
            Some(_)                 => self.size.number_huge,
        }
    }

    fn unit(&self, prefix: Option<number_prefix::Prefix>) -> Style {
        use number_prefix::Prefix::*;
        match prefix {
            None                    => self.size.unit_byte,
            Some(Kilo) | Some(Kibi) => self.size.unit_kilo,
            Some(Mega) | Some(Mibi) => self.size.unit_mega,
            Some(Giga) | Some(Gibi) => self.size.unit_giga,
            Some(_)                 => self.size.unit_huge,
        }
    }

    fn no_size(&self) -> Style { self.punctuation }
    fn major(&self)   -> Style { self.size.major }
    fn comma(&self)   -> Style { self.punctuation }
    fn minor(&self)   -> Style { self.size.minor }
}

impl render::UserColours for Colours {
    fn you(&self)           -> Style { self.users.user_you }
    fn someone_else(&self)  -> Style { self.users.user_someone_else }
}

impl FileNameColours for Colours {
    fn normal_arrow(&self)        -> Style { self.punctuation }
    fn broken_symlink(&self)      -> Style { self.broken_symlink }
    fn broken_filename(&self)     -> Style { apply_overlay(self.broken_symlink, self.broken_path_overlay) }
    fn broken_control_char(&self) -> Style { apply_overlay(self.control_char,   self.broken_path_overlay) }
    fn control_char(&self)        -> Style { self.control_char }
    fn symlink_path(&self)        -> Style { self.symlink_path }
    fn executable_file(&self)     -> Style { self.filekinds.executable }
}
