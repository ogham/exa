use file::{File, GREY};
use self::FileType::*;

use std::old_io as io;

use ansi_term::Style;
use ansi_term::Style::Plain;
use ansi_term::Colour::{Red, Green, Yellow, Blue, Cyan, Fixed};

#[derive(PartialEq, Debug, Copy)]
pub enum FileType {
    Normal, Directory, Executable, Immediate, Compiled, Symlink, Special,
    Image, Video, Music, Lossless, Compressed, Document, Temp, Crypto,
}

static IMAGE_TYPES: &'static [&'static str] = &[
    "png", "jpeg", "jpg", "gif", "bmp", "tiff", "tif",
    "ppm", "pgm", "pbm", "pnm", "webp", "raw", "arw",
    "svg", "stl", "eps", "dvi", "ps", "cbr",
    "cbz", "xpm", "ico" ];

static VIDEO_TYPES: &'static [&'static str] = &[
    "avi", "flv", "m2v", "mkv", "mov", "mp4", "mpeg",
    "mpg", "ogm", "ogv", "vob", "wmv" ];

static MUSIC_TYPES: &'static [&'static str] = &[
    "aac", "m4a", "mp3", "ogg", "wma" ];

static MUSIC_LOSSLESS: &'static [&'static str] = &[
    "alac", "ape", "flac", "wav" ];

static COMPRESSED_TYPES: &'static [&'static str] = &[
    "zip", "tar", "Z", "gz", "bz2", "a", "ar", "7z",
    "iso", "dmg", "tc", "rar", "par" ];

static DOCUMENT_TYPES: &'static [&'static str] = &[
    "djvu", "doc", "docx", "dvi", "eml", "eps", "fotd",
    "odp", "odt", "pdf", "ppt", "pptx", "rtf",
    "xls", "xlsx" ];

static TEMP_TYPES: &'static [&'static str] = &[
    "tmp", "swp", "swo", "swn", "bak" ];

static CRYPTO_TYPES: &'static [&'static str] = &[
    "asc", "gpg", "sig", "signature", "pgp" ];

static COMPILED_TYPES: &'static [&'static str] = &[
    "class", "elc", "hi", "o", "pyc" ];

static BUILD_TYPES: &'static [&'static str] = &[
    "Makefile", "Cargo.toml", "SConstruct", "CMakeLists.txt",
    "build.gradle", "Rakefile", "Gruntfile.js",
    "Gruntfile.coffee" ];

impl FileType {

    /// Get the `ansi_term::Style` that a file of this type should use.
    pub fn style(&self) -> Style {
        match *self {
            Normal     => Plain,
            Directory  => Blue.bold(),
            Symlink    => Cyan.normal(),
            Special    => Yellow.normal(),
            Executable => Green.bold(),
            Image      => Fixed(133).normal(),
            Video      => Fixed(135).normal(),
            Music      => Fixed(92).normal(),
            Lossless   => Fixed(93).normal(),
            Crypto     => Fixed(109).normal(),
            Document   => Fixed(105).normal(),
            Compressed => Red.normal(),
            Temp       => GREY.normal(),
            Immediate  => Yellow.bold().underline(),
            Compiled   => Fixed(137).normal(),
        }
    }
}

pub trait HasType {
    /// For a given file, find out what type it has.
    fn get_type(&self) -> FileType;
}

impl<'a> HasType for File<'a> {
    fn get_type(&self) -> FileType {

        match self.stat.kind {
            io::FileType::Directory    => return Directory,
            io::FileType::Symlink      => return Symlink,
            io::FileType::BlockSpecial => return Special,
            io::FileType::NamedPipe    => return Special,
            io::FileType::Unknown      => return Special,
            _ => {}
        }

        if self.name.starts_with("README") || BUILD_TYPES.contains(&&self.name[..]) {
            return Immediate;
        }
        else if let Some(ref ext) = self.ext {
            if IMAGE_TYPES.contains(&&ext[..]) {
                return Image;
            }
            else if VIDEO_TYPES.contains(&&ext[..]) {
                return Video;
            }
            else if MUSIC_TYPES.contains(&&ext[..]) {
                return Music;
            }
            else if MUSIC_LOSSLESS.contains(&&ext[..]) {
                return Lossless;
            }
            else if CRYPTO_TYPES.contains(&&ext[..]) {
                return Crypto;
            }
            else if DOCUMENT_TYPES.contains(&&ext[..]) {
                return Document;
            }
            else if COMPRESSED_TYPES.contains(&&ext[..]) {
                return Compressed;
            }
            else if self.is_tmpfile() || TEMP_TYPES.contains(&&ext[..]) {
                return Temp;
            }

            let source_files = self.get_source_files();
            if source_files.is_empty() {
                return Normal;
            }
            else if let Some(dir) = self.dir {
                if source_files.iter().any(|path| dir.contains(path)) {
                    return Temp;
                }
            }
            else if COMPILED_TYPES.contains(&&ext[..]) {
                return Compiled;
            }
        }

        return Normal;  // no filetype
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use file::test::{dummy_stat, new_file};

    #[test]
    fn lowercase() {
        let file = new_file(dummy_stat(), "/barracks.wav");
        assert_eq!(FileType::Lossless, file.get_type())
    }

    #[test]
    fn uppercase() {
        let file = new_file(dummy_stat(), "/BARRACKS.WAV");
        assert_eq!(FileType::Lossless, file.get_type())
    }

    #[test]
    fn cargo() {
        let file = new_file(dummy_stat(), "/Cargo.toml");
        assert_eq!(FileType::Immediate, file.get_type())
    }

    #[test]
    fn not_cargo() {
        let file = new_file(dummy_stat(), "/cargo.toml");
        assert_eq!(FileType::Normal, file.get_type())
    }

}
