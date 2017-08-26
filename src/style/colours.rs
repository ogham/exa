use ansi_term::Style;
use ansi_term::Colour::{Red, Green, Yellow, Blue, Cyan, Purple, Fixed};

use output::render;
use output::file_name::Colours as FileNameColours;

use style::lsc::Pair;


#[derive(Debug, Default, PartialEq)]
pub struct Colours {
    pub colourful: bool,
    pub scale: bool,

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

    pub symlink_path:     Style,
    pub broken_arrow:     Style,
    pub broken_filename:  Style,
    pub control_char:     Style,
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
    pub numbers: Style,
    pub unit: Style,

    pub major: Style,
    pub minor: Style,

    pub scale_byte: Style,
    pub scale_kilo: Style,
    pub scale_mega: Style,
    pub scale_giga: Style,
    pub scale_huge: Style,
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
}

impl Colours {
    pub fn plain() -> Colours {
        Colours::default()
    }

    pub fn colourful(scale: bool) -> Colours {
        Colours {
            colourful: true,
            scale: scale,

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

            size: Size {
                numbers:  Green.bold(),
                unit:     Green.normal(),

                major:  Green.bold(),
                minor:  Green.normal(),

                scale_byte: Fixed(118).normal(),
                scale_kilo: Fixed(190).normal(),
                scale_mega: Fixed(226).normal(),
                scale_giga: Fixed(220).normal(),
                scale_huge: Fixed(214).normal(),
            },

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
            },

            punctuation:  Fixed(244).normal(),
            date:         Blue.normal(),
            inode:        Purple.normal(),
            blocks:       Cyan.normal(),
            header:       Style::default().underline(),

            symlink_path:     Cyan.normal(),
            broken_arrow:     Red.normal(),
            broken_filename:  Red.underline(),
            control_char:     Red.normal(),
        }
    }
}


impl Colours {
    pub fn set_ls(&mut self, pair: &Pair) {
        match pair.key {
            "di" => self.filekinds.directory    = pair.to_style(),
            "ex" => self.filekinds.executable   = pair.to_style(),
            "fi" => self.filekinds.normal       = pair.to_style(),
            "pi" => self.filekinds.pipe         = pair.to_style(),
            "so" => self.filekinds.socket       = pair.to_style(),
            "bd" => self.filekinds.block_device = pair.to_style(),
            "cd" => self.filekinds.char_device  = pair.to_style(),
            "ln" => self.filekinds.symlink      = pair.to_style(),
            "or" => self.broken_arrow           = pair.to_style(),
            "mi" => self.broken_filename        = pair.to_style(),
             _   => {/* don’t change anything */},
        }
    }

    pub fn set_exa(&mut self, pair: &Pair) {
        match pair.key {
            "di" => self.filekinds.directory      = pair.to_style(),
            "ex" => self.filekinds.executable     = pair.to_style(),
            "fi" => self.filekinds.normal         = pair.to_style(),
            "pi" => self.filekinds.pipe           = pair.to_style(),
            "so" => self.filekinds.socket         = pair.to_style(),
            "bd" => self.filekinds.block_device   = pair.to_style(),
            "cd" => self.filekinds.char_device    = pair.to_style(),
            "ln" => self.filekinds.symlink        = pair.to_style(),
            "or" => self.broken_arrow             = pair.to_style(),
            "mi" => self.broken_filename          = pair.to_style(),

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

            "sn" => self.size.numbers             = pair.to_style(),
            "sb" => self.size.unit                = pair.to_style(),
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

             _   => {/* still don’t change anything */},
        }
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
    fn new(&self)           -> Style { self.git.new }
    fn modified(&self)      -> Style { self.git.modified }
    fn deleted(&self)       -> Style { self.git.deleted }
    fn renamed(&self)       -> Style { self.git.renamed }
    fn type_change(&self)   -> Style { self.git.typechange }
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
    fn size(&self, size: u64)  -> Style {
        if self.scale {
            if size < 1024 {
                self.size.scale_byte
            }
            else if size < 1024 * 1024 {
                self.size.scale_kilo
            }
            else if size < 1024 * 1024 * 1024 {
                self.size.scale_mega
            }
            else if size < 1024 * 1024 * 1024 * 1024 {
                self.size.scale_giga
            }
            else {
                self.size.scale_huge
            }
        }
        else {
            self.size.numbers
        }
    }

    fn unit(&self)    -> Style { self.size.unit }
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
    fn broken_arrow(&self)    -> Style { self.broken_arrow }
    fn broken_filename(&self) -> Style { self.broken_filename }
    fn normal_arrow(&self)    -> Style { self.punctuation }
    fn control_char(&self)    -> Style { self.control_char }
    fn symlink_path(&self)    -> Style { self.symlink_path }
    fn executable_file(&self) -> Style { self.filekinds.executable }
}

