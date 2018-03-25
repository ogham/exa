use std::io::{Write, Result as IOResult};

use term_grid as tg;

use fs::File;
use style::Colours;
use output::file_name::FileStyle;
use output::cell::DisplayWidth;


#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Options {
    pub across: bool,
    pub console_width: usize,
}

impl Options {
    pub fn direction(&self) -> tg::Direction {
        if self.across { tg::Direction::LeftToRight }
                  else { tg::Direction::TopToBottom }
    }
}


pub struct Render<'a> {
    pub files: Vec<File<'a>>,
    pub colours: &'a Colours,
    pub style: &'a FileStyle,
    pub opts: &'a Options,
}

impl<'a> Render<'a> {
    pub fn render<W: Write>(&self, w: &mut W) -> IOResult<()> {
        let mut grid = tg::Grid::new(tg::GridOptions {
            direction:  self.opts.direction(),
            filling:    tg::Filling::Spaces(2),
        });

        grid.reserve(self.files.len());

        for file in self.files.iter() {
            let file_icon = icon(&file);
            let painted_icon = self.style.exts
                .colour_file(&file)
                .map_or(file_icon.to_string(), |c| { c.paint(format!("{}", file_icon)).to_string() });
            let filename = self.style.for_file(file, self.colours).paint();
            let width = DisplayWidth::from(2) + filename.width();

            grid.add(tg::Cell {
                contents:  format!("{} {}", painted_icon, filename.strings().to_string()),
                width:     *width,
            });
        }

        if let Some(display) = grid.fit_into_width(self.opts.console_width) {
            write!(w, "{}", display)
        }
        else {
            // File names too long for a grid - drop down to just listing them!
            // This isn’t *quite* the same as the lines view, which also
            // displays full link paths.
            for file in self.files.iter() {
                let name_cell = self.style.for_file(file, self.colours).paint();
                writeln!(w, "{}", name_cell.strings())?;
            }
            Ok(())
        }
    }
}

fn icon(file: &File) -> char {
    if file.is_directory() { '\u{f115}' }
    else { 
        // possible unnecessary clone
        if let Some(ext) = file.ext.clone() {
            match ext.as_str() {
                "ai" => '\u{e7b4}',
                "android" => '\u{e70e}',
                "apple" => '\u{f179}',
                "audio" => '\u{f001}',
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
                "image" => '\u{f1c5}',
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
                "windows" => '\u{f17a}',
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
