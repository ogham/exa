use ansi_term::Style;
use ansi_term::Colour::{Red, Green, Yellow, Blue, Cyan, Purple, Fixed};


#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Colours {
    pub scale: bool,

    pub filetypes:  FileTypes,
    pub perms:      Permissions,
    pub size:       Size,
    pub users:      Users,
    pub links:      Links,
    pub git:        Git,
    pub date:       Date,

    pub punctuation:  Style,
    pub inode:        Style,
    pub blocks:       Style,
    pub header:       Style,

    pub symlink_path:     Style,
    pub broken_arrow:     Style,
    pub broken_filename:  Style,
    pub control_char:     Style,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FileTypes {
    pub normal: Style,
    pub directory: Style,
    pub symlink: Style,
    pub pipe: Style,
    pub device: Style,
    pub socket: Style,
    pub special: Style,
    pub executable: Style,
    pub image: Style,
    pub video: Style,
    pub music: Style,
    pub lossless: Style,
    pub crypto: Style,
    pub document: Style,
    pub compressed: Style,
    pub temp: Style,
    pub immediate: Style,
    pub compiled: Style,
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
pub struct Date {
    pub time_today: Style,
    pub time_yesterday: Style,
    pub time_week: Style,
    pub time_month: Style,
    pub time_year: Style,
    pub time_past: Style,
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
            scale: scale,

            filetypes: FileTypes {
                normal:      Style::default(),
                directory:   Blue.bold(),
                symlink:     Cyan.normal(),
                pipe:        Yellow.normal(),
                device:      Yellow.bold(),
                socket:      Red.bold(),
                special:     Yellow.normal(),
                executable:  Green.bold(),
                image:       Fixed(133).normal(),
                video:       Fixed(135).normal(),
                music:       Fixed(92).normal(),
                lossless:    Fixed(93).normal(),
                crypto:      Fixed(109).normal(),
                document:    Fixed(105).normal(),
                compressed:  Red.normal(),
                temp:        Fixed(244).normal(),
                immediate:   Yellow.bold().underline(),
                compiled:    Fixed(137).normal(),
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

            date: Date {
                time_today:         Fixed(56).normal(),
                time_yesterday:     Fixed(62).normal(),
                time_week:          Fixed(68).normal(),
                time_month:         Fixed(74).normal(),
                time_year:          Fixed(80).normal(),
                time_past:          Fixed(86).normal(),
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
            inode:        Purple.normal(),
            blocks:       Cyan.normal(),
            header:       Style::default().underline(),

            symlink_path:     Cyan.normal(),
            broken_arrow:     Red.normal(),
            broken_filename:  Red.underline(),
            control_char:     Red.normal(),
        }
    }

    pub fn file_size(&self, size: u64) -> Style {
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

    pub fn stamp_age (&self, age: i64) -> Style {
        if age < 60*60*24 {
            self.date.time_today
        }
        else if age < 60*60*24*2 {
            self.date.time_yesterday
        }
        else if age < 60*60*24*7 {
            self.date.time_week
        }
        else if age < 60*60*24*30 {
            self.date.time_month
        }
        else if age < 60*60*24*365 {
            self.date.time_year
        }
        else {
            self.date.time_past
        }
    }
}
