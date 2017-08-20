use ansi_term::{ANSIString, Style};

use fs::fields as f;


impl f::Type {
    pub fn render<C: Colours>(&self, colours: &C) -> ANSIString<'static> {
        match *self {
            f::Type::File        => colours.normal().paint("."),
            f::Type::Directory   => colours.directory().paint("d"),
            f::Type::Pipe        => colours.pipe().paint("|"),
            f::Type::Link        => colours.symlink().paint("l"),
            f::Type::CharDevice  => colours.device().paint("c"),
            f::Type::BlockDevice => colours.device().paint("b"),
            f::Type::Socket      => colours.socket().paint("s"),
            f::Type::Special     => colours.special().paint("?"),
        }
    }
}


pub trait Colours {
    fn normal(&self) -> Style;
    fn directory(&self) -> Style;
    fn pipe(&self) -> Style;
    fn symlink(&self) -> Style;
    fn device(&self) -> Style;
    fn socket(&self) -> Style;
    fn special(&self) -> Style;
}
