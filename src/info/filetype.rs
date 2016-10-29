//! Tests for various types of file (video, image, compressed, etc).
//!
//! Currently this is dependent on the file’s name and extension, because
//! those are the only metadata that we have access to without reading the
//! file’s contents.

use fs::File;


impl<'a> File<'a> {

    /// An “immediate” file is something that can be run or activated somehow
    /// in order to kick off the build of a project. It’s usually only present
    /// in directories full of source code.
    pub fn is_immediate(&self) -> bool {
        self.name.starts_with("README") || self.name_is_one_of( &[
            "Makefile", "Cargo.toml", "SConstruct", "CMakeLists.txt",
            "build.gradle", "Rakefile", "Gruntfile.js",
            "Gruntfile.coffee",
        ])
    }

    pub fn is_image(&self) -> bool {
        self.extension_is_one_of( &[
            "png", "jpeg", "jpg", "gif", "bmp", "tiff", "tif",
            "ppm", "pgm", "pbm", "pnm", "webp", "raw", "arw",
            "svg", "stl", "eps", "dvi", "ps", "cbr",
            "cbz", "xpm", "ico",
        ])
    }

    pub fn is_video(&self) -> bool {
        self.extension_is_one_of( &[
            "avi", "flv", "m2v", "mkv", "mov", "mp4", "mpeg",
            "mpg", "ogm", "ogv", "vob", "wmv",
        ])
    }

    pub fn is_music(&self) -> bool {
        self.extension_is_one_of( &[
            "aac", "m4a", "mp3", "ogg", "wma",
        ])
    }

    // Lossless music, rather than any other kind of data...
    pub fn is_lossless(&self) -> bool {
        self.extension_is_one_of( &[
            "alac", "ape", "flac", "wav",
        ])
    }

    pub fn is_crypto(&self) -> bool {
        self.extension_is_one_of( &[
            "asc", "enc", "gpg", "pgp", "sig", "signature", "pfx", "p12",
        ])
    }

    pub fn is_document(&self) -> bool {
        self.extension_is_one_of( &[
            "djvu", "doc", "docx", "dvi", "eml", "eps", "fotd",
            "odp", "odt", "pdf", "ppt", "pptx", "rtf",
            "xls", "xlsx",
        ])
    }

    pub fn is_compressed(&self) -> bool {
        self.extension_is_one_of( &[
            "zip", "tar", "Z", "gz", "bz2", "a", "ar", "7z",
            "iso", "dmg", "tc", "rar", "par", "tgz",
        ])
    }

    pub fn is_temp(&self) -> bool {
        self.name.ends_with("~")
            || (self.name.starts_with("#") && self.name.ends_with("#"))
            || self.extension_is_one_of( &[ "tmp", "swp", "swo", "swn", "bak" ])
    }

    pub fn is_compiled(&self) -> bool {
        if self.extension_is_one_of( &[ "class", "elc", "hi", "o", "pyc" ]) {
            true
        }
        else if let Some(dir) = self.dir {
            self.get_source_files().iter().any(|path| dir.contains(path))
        }
        else {
            false
        }
    }
}


#[cfg(broken_test)]
mod test {
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
