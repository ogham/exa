//! Tests for various types of file (video, image, compressed, etc).
//!
//! Currently this is dependent on the file’s name and extension, because
//! those are the only metadata that we have access to without reading the
//! file’s contents.

use fs::File;


#[derive(Debug)]
pub struct FileExtensions;

impl FileExtensions {

    /// An “immediate” file is something that can be run or activated somehow
    /// in order to kick off the build of a project. It’s usually only present
    /// in directories full of source code.
    pub fn is_immediate(&self, file: &File) -> bool {
        file.name.starts_with("README") || file.name_is_one_of( &[
            "Makefile", "Cargo.toml", "SConstruct", "CMakeLists.txt",
            "build.gradle", "Rakefile", "Gruntfile.js",
            "Gruntfile.coffee",
        ])
    }

    pub fn is_image(&self, file: &File) -> bool {
        file.extension_is_one_of( &[
            "png", "jpeg", "jpg", "gif", "bmp", "tiff", "tif",
            "ppm", "pgm", "pbm", "pnm", "webp", "raw", "arw",
            "svg", "stl", "eps", "dvi", "ps", "cbr",
            "cbz", "xpm", "ico", "cr2", "orf", "nef",
        ])
    }

    pub fn is_video(&self, file: &File) -> bool {
        file.extension_is_one_of( &[
            "avi", "flv", "m2v", "mkv", "mov", "mp4", "mpeg",
            "mpg", "ogm", "ogv", "vob", "wmv", "webm", "m2ts",
            "ts",
        ])
    }

    pub fn is_music(&self, file: &File) -> bool {
        file.extension_is_one_of( &[
            "aac", "m4a", "mp3", "ogg", "wma", "mka", "opus",
        ])
    }

    // Lossless music, rather than any other kind of data...
    pub fn is_lossless(&self, file: &File) -> bool {
        file.extension_is_one_of( &[
            "alac", "ape", "flac", "wav",
        ])
    }

    pub fn is_crypto(&self, file: &File) -> bool {
        file.extension_is_one_of( &[
            "asc", "enc", "gpg", "pgp", "sig", "signature", "pfx", "p12",
        ])
    }

    pub fn is_document(&self, file: &File) -> bool {
        file.extension_is_one_of( &[
            "djvu", "doc", "docx", "dvi", "eml", "eps", "fotd",
            "odp", "odt", "pdf", "ppt", "pptx", "rtf",
            "xls", "xlsx",
        ])
    }

    pub fn is_compressed(&self, file: &File) -> bool {
        file.extension_is_one_of( &[
            "zip", "tar", "Z", "z", "gz", "bz2", "a", "ar", "7z",
            "iso", "dmg", "tc", "rar", "par", "tgz", "xz", "txz",
            "lzma", "deb", "rpm"
        ])
    }

    pub fn is_temp(&self, file: &File) -> bool {
        file.name.ends_with('~')
            || (file.name.starts_with('#') && file.name.ends_with('#'))
            || file.extension_is_one_of( &[ "tmp", "swp", "swo", "swn", "bak" ])
    }

    pub fn is_compiled(&self, file: &File) -> bool {
        if file.extension_is_one_of( &[ "class", "elc", "hi", "o", "pyc" ]) {
            true
        }
        else if let Some(dir) = file.parent_dir {
            file.get_source_files().iter().any(|path| dir.contains(path))
        }
        else {
            false
        }
    }
}
