use colours::{Plain, Style, Black, Red, Green, Yellow, Blue, Purple, Cyan, Fixed};
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

    pub fn from_file(file: &File) -> FileType {
        if file.stat.kind == io::TypeDirectory {
            return Directory;
        }
        else if file.stat.perm.contains(io::UserExecute) {
            return Executable;
        }
        else if file.ext.is_some() {
            let ext = file.ext.unwrap();
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
            else if file.is_tmpfile() || TEMP_TYPES.iter().any(|&s| s == ext) {
                return Temp;
            }
        }

        if file.name.starts_with("README") {
            return Immediate;
        }

        let source_files = file.get_source_files();
        if source_files.len() == 0 {
            let source_files_usual = file.get_source_files_usual();
            if source_files_usual.iter().any(|path| file.dir.contains(path)) {
                Temp
            }
            else {
                Normal
            }
        }
        else if source_files.iter().any(|path| file.dir.contains(path)) {
            Temp
        }
        else {
            Compiled
        }
    }
}

