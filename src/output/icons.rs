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
    format!("{} ", painted)
}

fn icon(file: &File) -> char {
    let extensions = Box::new(FileExtensions);
    if file.is_directory() { '\u{f115}' }
    else if let Some(icon) = extensions.icon_file(file) { icon }
    else { 
        if let Some(ext) = file.ext.as_ref() {
            match ext.as_str() {
                "ai" => '\u{e7b4}',
                "android" => '\u{e70e}',
                "apple" => '\u{f179}',
                "avro" => '\u{e60b}',
                "c" => '\u{e61e}',
                "clj" => '\u{e768}',
                "coffee" => '\u{f0f4}',
                "conf" => '\u{e615}',
                "cpp" => '\u{e61d}',
                "css" => '\u{e749}',
                "d" => '\u{e7af}',
                "dart" => '\u{e798}',
                "db" => '\u{f1c0}',
                "diff" => '\u{f440}',
                "doc" => '\u{f1c2}',
                "ebook" => '\u{e28b}',
                "env" => '\u{f462}',
                "epub" => '\u{e28a}',
                "erl" => '\u{e7b1}',
                "font" => '\u{f031}',
                "gform" => '\u{f298}',
                "git" => '\u{f1d3}',
                "go" => '\u{e626}',
                "hs" => '\u{e777}',
                "html" => '\u{f13b}',
                "iml" => '\u{e7b5}',
                "java" => '\u{e204}',
                "js" => '\u{e74e}',
                "json" => '\u{e60b}',
                "jsx" => '\u{e7ba}',
                "less" => '\u{e758}',
                "log" => '\u{f18d}',
                "lua" => '\u{e620}',
                "md" => '\u{f48a}',
                "mustache" => '\u{e60f}',
                "npmignore" => '\u{e71e}',
                "pdf" => '\u{f1c1}',
                "php" => '\u{e73d}',
                "pl" => '\u{e769}',
                "ppt" => '\u{f1c4}',
                "psd" => '\u{e7b8}',
                "py" => '\u{e606}',
                "r" => '\u{f25d}',
                "rb" => '\u{e21e}',
                "rdb" => '\u{e76d}',
                "rs" => '\u{e7a8}',
                "rss" => '\u{f09e}',
                "rubydoc" => '\u{e73b}',
                "sass" => '\u{e603}',
                "scala" => '\u{e737}',
                "shell" => '\u{f489}',
                "sqlite3" => '\u{e7c4}',
                "styl" => '\u{e600}',
                "tex" => '\u{e600}',
                "ts" => '\u{e628}',
                "twig" => '\u{e61c}',
                "txt" => '\u{f15c}',
                "video" => '\u{f03d}',
                "vim" => '\u{e62b}',
                "xls" => '\u{f1c3}',
                "xml" => '\u{e619}',
                "yml" => '\u{f481}',
                "zip" => '\u{f410}',
                _ => '\u{f15b}'
            }
        } else {
            '\u{f15b}'
        }
    }
}
