//! Tests for various types of file (video, image, compressed, etc).
//!
//! Currently this is dependent on the file’s name and extension, because
//! those are the only metadata that we have access to without reading the
//! file’s contents.

use ansi_term::Style;

use crate::fs::File;
use crate::output::icons::FileIcon;
use crate::theme::FileColours;


#[derive(Debug, Default, PartialEq)]
pub struct FileExtensions;

impl FileExtensions {

    /// An “immediate” file is something that can be run or activated somehow
    /// in order to kick off the build of a project. It’s usually only present
    /// in directories full of source code.
    fn is_immediate(&self, file: &File<'_>) -> bool {
        file.name.to_lowercase().starts_with("readme") ||
        file.name.ends_with(".ninja") ||
        file.name_is_one_of( &[
            "Makefile", "Cargo.toml", "SConstruct", "CMakeLists.txt",
            "build.gradle", "pom.xml", "Rakefile", "package.json", "Gruntfile.js",
            "Gruntfile.coffee", "BUILD", "BUILD.bazel", "WORKSPACE", "build.xml",
            "webpack.config.js", "meson.build", "composer.json", "RoboFile.php", "PKGBUILD",
            "Justfile", "Procfile", "Dockerfile", "Containerfile", "Vagrantfile", "Brewfile",
            "Gemfile", "Pipfile", "build.sbt", "mix.exs", "bsconfig.json", "tsconfig.json",
        ])
    }

    fn is_image(&self, file: &File<'_>) -> bool {
        file.extension_is_one_of( &[
            "png", "jpeg", "jpg", "gif", "bmp", "tiff", "tif",
            "ppm", "pgm", "pbm", "pnm", "webp", "raw", "arw",
            "svg", "stl", "eps", "dvi", "ps", "cbr", "jpf",
            "cbz", "xpm", "ico", "cr2", "orf", "nef", "heif",
        ])
    }

    fn is_video(&self, file: &File<'_>) -> bool {
        file.extension_is_one_of( &[
            "avi", "flv", "m2v", "m4v", "mkv", "mov", "mp4", "mpeg",
            "mpg", "ogm", "ogv", "vob", "wmv", "webm", "m2ts", "heic",
        ])
    }

    fn is_music(&self, file: &File<'_>) -> bool {
        file.extension_is_one_of( &[
            "aac", "m4a", "mp3", "ogg", "wma", "mka", "opus",
        ])
    }

    // Lossless music, rather than any other kind of data...
    fn is_lossless(&self, file: &File<'_>) -> bool {
        file.extension_is_one_of( &[
            "alac", "ape", "flac", "wav",
        ])
    }

    fn is_crypto(&self, file: &File<'_>) -> bool {
        file.extension_is_one_of( &[
            "asc", "enc", "gpg", "pgp", "sig", "signature", "pfx", "p12",
        ])
    }

    fn is_document(&self, file: &File<'_>) -> bool {
        file.extension_is_one_of( &[
            "djvu", "doc", "docx", "dvi", "eml", "eps", "fotd", "key",
            "keynote", "numbers", "odp", "odt", "pages", "pdf", "ppt",
            "pptx", "rtf", "xls", "xlsx",
        ])
    }

    fn is_compressed(&self, file: &File<'_>) -> bool {
        file.extension_is_one_of( &[
            "zip", "tar", "Z", "z", "gz", "bz2", "a", "ar", "7z",
            "iso", "dmg", "tc", "rar", "par", "tgz", "xz", "txz",
            "lz", "tlz", "lzma", "deb", "rpm", "zst",
        ])
    }

    fn is_temp(&self, file: &File<'_>) -> bool {
        file.name.ends_with('~')
            || (file.name.starts_with('#') && file.name.ends_with('#'))
            || file.extension_is_one_of( &[ "tmp", "swp", "swo", "swn", "bak", "bk" ])
    }

    fn is_compiled(&self, file: &File<'_>) -> bool {
        if file.extension_is_one_of( &[ "class", "elc", "hi", "o", "pyc", "zwc", "ko" ]) {
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

impl FileColours for FileExtensions {
    fn colour_file(&self, file: &File<'_>) -> Option<Style> {
        use ansi_term::Colour::*;

        Some(match file {
            f if self.is_temp(f)        => Fixed(244).normal(),
            f if self.is_immediate(f)   => Yellow.bold().underline(),
            f if self.is_image(f)       => Fixed(133).normal(),
            f if self.is_video(f)       => Fixed(135).normal(),
            f if self.is_music(f)       => Fixed(92).normal(),
            f if self.is_lossless(f)    => Fixed(93).normal(),
            f if self.is_crypto(f)      => Fixed(109).normal(),
            f if self.is_document(f)    => Fixed(105).normal(),
            f if self.is_compressed(f)  => Red.normal(),
            f if self.is_compiled(f)    => Fixed(137).normal(),
            _                           => return None,
        })
    }
}

impl FileIcon for FileExtensions {
    fn icon_file(&self, file: &File<'_>) -> Option<char> {
        use crate::output::icons::Icons;

        if self.is_music(file) || self.is_lossless(file) {
            Some(Icons::Audio.value())
        }
        else if self.is_image(file) {
            Some(Icons::Image.value())
        }
        else if self.is_video(file) {
            Some(Icons::Video.value())
        }
        else {
            None
        }
    }
}
