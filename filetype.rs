use colours::{Plain, Style, Red, Green, Yellow, Blue, Fixed};
use file::File;
use std::io;

pub enum FileType {
    Normal, Directory, Executable, Immediate, Compiled,
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
    "aac", "m4a", "mp3", "ogg" ];

static MUSIC_LOSSLESS: &'static [&'static str] = &[
    "alac", "ape", "flac", "wav" ];

static COMPRESSED_TYPES: &'static [&'static str] = &[
    "zip", "tar", "Z", "gz", "bz2", "a", "ar", "7z",
    "iso", "dmg", "tc", "rar", "par" ];

static DOCUMENT_TYPES: &'static [&'static str] = &[
    "djvu", "doc", "docx", "eml", "eps", "odp", "ods",
    "odt", "pdf", "ppt", "pptx", "xls", "xlsx" ];

static TEMP_TYPES: &'static [&'static str] = &[
    "tmp", "swp", "swo", "swn", "bak" ];

static CRYPTO_TYPES: &'static [&'static str] = &[
    "asc", "gpg", "sig", "signature", "pgp" ];

static COMPILED_TYPES: &'static [&'static str] = &[
    "class", "elc", "hi", "o", "pyc" ];

impl FileType {
    pub fn style(&self) -> Style {
        match *self {
            Normal => Plain,
            Directory => Blue.bold(),
            Executable => Green.bold(),
            Image => Fixed(133).normal(),
            Video => Fixed(135).normal(),
            Music => Fixed(92).normal(),
            Lossless => Fixed(93).normal(),
            Crypto => Fixed(109).normal(),
            Document => Fixed(105).normal(),
            Compressed => Red.normal(),
            Temp => Fixed(244).normal(),
            Immediate => Yellow.bold().underline(),
            Compiled => Fixed(137).normal(),
        }
    }
}

pub trait HasType {
    fn get_type(&self) -> FileType;
}

impl<'a> HasType for File<'a> {
    fn get_type(&self) -> FileType {
        if self.stat.kind == io::TypeDirectory {
            return Directory;
        }
        else if self.stat.perm.contains(io::UserExecute) {
            return Executable;
        }
        else if self.name.starts_with("README") {
            return Immediate;
        }
        else if self.ext.is_some() {
            let ext = self.ext.unwrap();
            if IMAGE_TYPES.iter().any(|&s| s == ext) {
                return Image;
            }
            else if VIDEO_TYPES.iter().any(|&s| s == ext) {
                return Video;
            }
            else if MUSIC_TYPES.iter().any(|&s| s == ext) {
                return Music;
            }
            else if MUSIC_LOSSLESS.iter().any(|&s| s == ext) {
                return Lossless;
            }
            else if CRYPTO_TYPES.iter().any(|&s| s == ext) {
                return Crypto;
            }
            else if DOCUMENT_TYPES.iter().any(|&s| s == ext) {
                return Document;
            }
            else if COMPRESSED_TYPES.iter().any(|&s| s == ext) {
                return Compressed;
            }
            else if self.is_tmpfile() || TEMP_TYPES.iter().any(|&s| s == ext) {
                return Temp;
            }
            
            let source_files = self.get_source_files();
            if source_files.len() == 0 {
                return Normal;
            }
            else if source_files.iter().any(|path| self.dir.contains(path)) {
                return Temp;
            }
            else {
                if COMPILED_TYPES.iter().any(|&s| s == ext) {
                    return Compiled;
                }
                else {
                    return Normal;
                }
            }
        }
        return Normal;  // no filetype
    }
}

