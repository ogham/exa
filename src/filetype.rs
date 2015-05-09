use ansi_term::Style;

use file::File;
use colours::Colours;

pub fn file_colour(colours: &Colours, file: &File) -> Style {
    match file {
        f if f.is_directory()        => colours.directory,
        f if f.is_executable_file()  => colours.executable,
        f if f.is_link()             => colours.symlink,
        f if !f.is_file()            => colours.special,
        f if f.is_immediate()        => colours.immediate,
        f if f.is_image()            => colours.image,
        f if f.is_video()            => colours.video,
        f if f.is_music()            => colours.music,
        f if f.is_lossless()         => colours.lossless,
        f if f.is_crypto()           => colours.crypto,
        f if f.is_document()         => colours.document,
        f if f.is_compressed()       => colours.compressed,
        f if f.is_temp()             => colours.temp,
        f if f.is_compiled()         => colours.compiled,
        _                            => colours.normal,
    }
}

trait FileTypes {
    fn is_immediate(&self) -> bool;
    fn is_image(&self) -> bool;
    fn is_video(&self) -> bool;
    fn is_music(&self) -> bool;
    fn is_lossless(&self) -> bool;
    fn is_crypto(&self) -> bool;
    fn is_document(&self) -> bool;
    fn is_compressed(&self) -> bool;
    fn is_temp(&self) -> bool;
    fn is_compiled(&self) -> bool;
}

impl<'_> FileTypes for File<'_> {
    fn is_immediate(&self) -> bool {
        self.name.starts_with("README") || self.name_is_one_of( &[
            "Makefile", "Cargo.toml", "SConstruct", "CMakeLists.txt",
            "build.gradle", "Rakefile", "Gruntfile.js",
            "Gruntfile.coffee",
        ])
	}

    fn is_image(&self) -> bool {
        self.extension_is_one_of( &[
            "png", "jpeg", "jpg", "gif", "bmp", "tiff", "tif",
            "ppm", "pgm", "pbm", "pnm", "webp", "raw", "arw",
            "svg", "stl", "eps", "dvi", "ps", "cbr",
            "cbz", "xpm", "ico",
        ])
	}

    fn is_video(&self) -> bool {
        self.extension_is_one_of( &[
            "avi", "flv", "m2v", "mkv", "mov", "mp4", "mpeg",
            "mpg", "ogm", "ogv", "vob", "wmv",
        ])
	}

    fn is_music(&self) -> bool {
        self.extension_is_one_of( &[
            "aac", "m4a", "mp3", "ogg", "wma",
        ])
	}

    fn is_lossless(&self) -> bool {
        self.extension_is_one_of( &[
            "alac", "ape", "flac", "wav",
        ])
	}

    fn is_crypto(&self) -> bool {
        self.extension_is_one_of( &[
            "zip", "tar", "Z", "gz", "bz2", "a", "ar", "7z",
            "iso", "dmg", "tc", "rar", "par",
        ])
	}

    fn is_document(&self) -> bool {
        self.extension_is_one_of( &[
            "djvu", "doc", "docx", "dvi", "eml", "eps", "fotd",
            "odp", "odt", "pdf", "ppt", "pptx", "rtf",
            "xls", "xlsx",
        ])
	}

    fn is_compressed(&self) -> bool {
        self.extension_is_one_of( &[
            "zip", "tar", "Z", "gz", "bz2", "a", "ar", "7z",
            "iso", "dmg", "tc", "rar", "par"
        ])
	}

    fn is_temp(&self) -> bool {
        self.name.ends_with("~")
            || (self.name.starts_with("#") && self.name.ends_with("#"))
            || self.extension_is_one_of( &[ "tmp", "swp", "swo", "swn", "bak" ])
	}

    fn is_compiled(&self) -> bool {
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
