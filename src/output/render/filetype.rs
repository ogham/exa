use ansi_term::{ANSIString, Style};

use fs::fields as f;


impl f::Type {
    pub fn render<C: Colours>(&self, colours: &C) -> ANSIString<'static> {
        match *self {
            f::Type::File        => colours.normal().paint("."),
            f::Type::Directory   => colours.directory().paint("d"),
            f::Type::Pipe        => colours.pipe().paint("|"),
            f::Type::Link        => colours.symlink().paint("l"),
            f::Type::BlockDevice => colours.block_device().paint("b"),
            f::Type::CharDevice  => colours.char_device().paint("c"),
            f::Type::Socket      => colours.socket().paint("s"),
            f::Type::Special     => colours.special().paint("?"),
        }
    }
}


pub trait Colours {
    fn normal(&self) -> Style;
    fn bundle(&self) -> Style;
    fn directory(&self) -> Style;
    fn pipe(&self) -> Style;
    fn symlink(&self) -> Style;
    fn block_device(&self) -> Style;
    fn char_device(&self) -> Style;
    fn socket(&self) -> Style;
    fn special(&self) -> Style;
}
