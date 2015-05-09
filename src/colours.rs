use ansi_term::Style;
use ansi_term::Style::Plain;
use ansi_term::Colour::{Red, Green, Yellow, Blue, Cyan, Fixed};

use file::GREY;

use std::default::Default;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Colours {
    pub normal: Style,
    pub directory: Style,
	pub symlink: Style,
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

impl Colours {
    pub fn plain() -> Colours {
        Colours::default()
    }

    pub fn colourful() -> Colours {
        Colours {
            normal:      Plain,
            directory:   Blue.bold(),
            symlink:     Cyan.normal(),
            special:     Yellow.normal(),
            executable:  Green.bold(),
            image:       Fixed(133).normal(),
            video:       Fixed(135).normal(),
            music:       Fixed(92).normal(),
            lossless:    Fixed(93).normal(),
            crypto:      Fixed(109).normal(),
            document:    Fixed(105).normal(),
            compressed:  Red.normal(),
            temp:        GREY.normal(),
            immediate:   Yellow.bold().underline(),
            compiled:    Fixed(137).normal(),
        }
    }
}
