use ansi_term::{ANSIString, Style};

use crate::{fs::fields as f, theme::LinkStyle};

impl f::Type {
    pub fn render<C: Colours>(self, colours: &C) -> ANSIString<'static> {
        match self {
            Self::File         => colours.normal().paint("."),
            Self::Directory    => colours.directory().paint("d"),
            Self::Pipe         => colours.pipe().paint("|"),
            Self::Link         => {
                match colours.symlink() {
                    LinkStyle::AnsiStyle(s) => s.paint("l"),
                    LinkStyle::Target => colours.normal().paint("l")
                }
            }
            Self::BlockDevice  => colours.block_device().paint("b"),
            Self::CharDevice   => colours.char_device().paint("c"),
            Self::Socket       => colours.socket().paint("s"),
            Self::Special      => colours.special().paint("?"),
        }
    }
}


pub trait Colours {
    fn normal(&self) -> Style;
    fn directory(&self) -> Style;
    fn pipe(&self) -> Style;
    fn symlink(&self) -> LinkStyle;
    fn block_device(&self) -> Style;
    fn char_device(&self) -> Style;
    fn socket(&self) -> Style;
    fn special(&self) -> Style;
}
