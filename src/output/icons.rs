use ansi_term::Style;

use crate::fs::File;
use crate::info::filetype::FileExtensions;
use crate::output::file_name::FileStyle;

pub trait FileIcon {
    fn icon_file(&self, file: &File) -> Option<char>;
}

pub enum Icons {
    Audio,
    Image,
    Video,
}

impl Icons {
    pub fn value(&self) -> char {
        match *self {
            Icons::Audio => '\u{f001}',
            Icons::Image => '\u{f1c5}',
            Icons::Video => '\u{f03d}',
        }
    }
}

pub fn painted_icon(file: &File, style: &FileStyle) -> String {
    let file_icon = icon(&file).to_string();
    let painted = style.exts
            .colour_file(&file)
            .map_or(file_icon.to_string(), |c| {
                // Remove underline from icon
                if c.is_underline {
                    match c.foreground {
                        Some(color) => Style::from(color).paint(file_icon).to_string(),
                        None => Style::default().paint(file_icon).to_string(),
                    }
                } else {
                    c.paint(file_icon).to_string()
                }
            });
    format!("{}  ", painted)
}

fn icon(file: &File) -> char {
    let extensions = Box::new(FileExtensions);
    if file.points_to_directory() { '\u{f115}' }
    else if let Some(icon) = extensions.icon_file(file) { icon }
    else if let Some(ext) = file.ext.as_ref() {
        match ext.as_str() {
            "ai"        => '\u{e7b4}',
            "android"   => '\u{e70e}',
            "apple"     => '\u{f179}',
            "avro"      => '\u{e60b}',
            "clj"       => '\u{e768}',
            "coffee"    => '\u{f0f4}',
            "cpp"       => '\u{e61d}',
            "hpp"       => '\u{e61d}',
            "c"         => '\u{e61e}',
            "h"         => '\u{e61e}',
            "cs"        => '\u{f81a}',
            "css"       => '\u{e749}',
            "d"         => '\u{e7af}',
            "dart"      => '\u{e798}',
            "db"        => '\u{f1c0}',
            "diff"      => '\u{f440}',
            "patch"     => '\u{f440}',
            "rtf"       => '\u{f1c2}',
            "doc"       => '\u{f1c2}',
            "docx"      => '\u{f1c2}',
            "odt"       => '\u{f1c2}',
            "ebook"     => '\u{e28b}',
            "env"       => '\u{f462}',
            "epub"      => '\u{e28a}',
            "erl"       => '\u{e7b1}',
            "font"      => '\u{f031}',
            "gform"     => '\u{f298}',
            "git"       => '\u{f1d3}',
            "go"        => '\u{e626}',
            "hs"        => '\u{e777}',
            "htm"       => '\u{f13b}',
            "html"      => '\u{f13b}',
            "xhtml"     => '\u{f13b}',
            "iml"       => '\u{e7b5}',
            "java"      => '\u{e204}',
            "js"        => '\u{e74e}',
            "mjs"       => '\u{e74e}',
            "json"      => '\u{e60b}',
            "jsx"       => '\u{e7ba}',
            "vue"       => '\u{fd42}',
            "node"      => '\u{f898}',
            "less"      => '\u{e758}',
            "log"       => '\u{f18d}',
            "lua"       => '\u{e620}',
            "md"        => '\u{f48a}',
            "markdown"  => '\u{f48a}',
            "mustache"  => '\u{e60f}',
            "npmignore" => '\u{e71e}',
            "pdf"       => '\u{f1c1}',
            "djvu"      => '\u{f02d}',
            "mobi"      => '\u{f02d}',
            "php"       => '\u{e73d}',
            "pl"        => '\u{e769}',
            "ppt"       => '\u{f1c4}',
            "pptx"      => '\u{f1c4}',
            "odp"       => '\u{f1c4}',
            "psd"       => '\u{e7b8}',
            "py"        => '\u{e606}',
            "r"         => '\u{f25d}',
            "rb"        => '\u{e21e}',
            "ru"        => '\u{e21e}',
            "erb"       => '\u{e21e}',
            "gem"       => '\u{e21e}',
            "rdb"       => '\u{e76d}',
            "rs"        => '\u{e7a8}',
            "rss"       => '\u{f09e}',
            "rubydoc"   => '\u{e73b}',
            "sass"      => '\u{e74b}',
            "stylus"    => '\u{e759}',
            "scala"     => '\u{e737}',
            "shell"     => '\u{f489}',
            "sqlite3"   => '\u{e7c4}',
            "styl"      => '\u{e600}',
            "latex"     => '\u{e600}',
            "tex"       => '\u{e600}',
            "ts"        => '\u{e628}',
            "tsx"       => '\u{e628}',
            "twig"      => '\u{e61c}',
            "txt"       => '\u{f15c}',
            "video"     => '\u{f03d}',
            "vim"       => '\u{e62b}',
            "xml"       => '\u{e619}',
            "yml"       => '\u{f481}',
            "yaml"      => '\u{f481}',
            "rar"       => '\u{f410}',
            "zip"       => '\u{f410}',
            "bz"        => '\u{f410}',
            "bz2"       => '\u{f410}',
            "xz"        => '\u{f410}',
            "taz"       => '\u{f410}',
            "tbz"       => '\u{f410}',
            "tbz2"      => '\u{f410}',
            "tz"        => '\u{f410}',
            "tar"       => '\u{f410}',
            "tzo"       => '\u{f410}',
            "lz"        => '\u{f410}',
            "lzh"       => '\u{f410}',
            "lzma"      => '\u{f410}',
            "lzo"       => '\u{f410}',
            "gz"        => '\u{f410}',
            "deb"       => '\u{e77d}',
            "rpm"       => '\u{e7bb}',
            "exe"       => '\u{e70f}',
            "msi"       => '\u{e70f}',
            "dll"       => '\u{e70f}',
            "cab"       => '\u{e70f}',
            "bat"       => '\u{e70f}',
            "cmd"       => '\u{e70f}',
            "sh"        => '\u{f489}',
            "bash"      => '\u{f489}',
            "zsh"       => '\u{f489}',
            "fish"      => '\u{f489}',
            "csh"       => '\u{f489}',
            "ini"       => '\u{e615}',
            "toml"      => '\u{e615}',
            "cfg"       => '\u{e615}',
            "conf"      => '\u{e615}',
            "apk"       => '\u{e70e}',
            "ttf"       => '\u{f031}',
            "woff"      => '\u{f031}',
            "woff2"     => '\u{f031}',
            "otf"       => '\u{f031}',
            "csv"       => '\u{f1c3}',
            "tsv"       => '\u{f1c3}',
            "xls"       => '\u{f1c3}',
            "xlsx"      => '\u{f1c3}',
            "ods"       => '\u{f1c3}',
            "so"        => '\u{f17c}',
            "sql"       => '\u{f1c0}',
            "jar"       => '\u{e256}',
            "jad"       => '\u{e256}',
            "class"     => '\u{e256}',
            "war"       => '\u{e256}',
            "groovy"    => '\u{e775}',
            "iso"       => '\u{e271}',
            "lock"      => '\u{f023}',
            "swift"     => '\u{e755}',
            "nix"       => '\u{f313}',
            _           => '\u{f016}'
        }
    } else {
        '\u{f016}'
    }
}
