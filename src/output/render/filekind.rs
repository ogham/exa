use ansi_term::{ANSIString, Style};

use crate::fs::fields as f;


impl f::Type {
    pub fn render<C: Colours>(self, colours: &C) -> ANSIString<'static> {
        match self {
            Self::File         => colours.normal().paint("."),
            Self::Directory    => colours.directory().paint("d"),
            Self::Pipe         => colours.pipe().paint("|"),
            Self::Link         => colours.symlink().paint("l"),
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
    fn symlink(&self) -> Style;
    fn block_device(&self) -> Style;
    fn char_device(&self) -> Style;
    fn socket(&self) -> Style;
    fn special(&self) -> Style;
}
