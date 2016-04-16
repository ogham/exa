use ansi_term::Style;

use fs::File;

pub use self::cell::{TextCell, TextCellContents, DisplayWidth};
pub use self::colours::Colours;
pub use self::details::Details;
pub use self::grid_details::GridDetails;
pub use self::grid::Grid;
pub use self::lines::Lines;

mod grid;
pub mod details;
mod lines;
mod grid_details;
pub mod column;
mod cell;
mod colours;
mod tree;


pub fn filename(file: &File, colours: &Colours, links: bool) -> TextCellContents {
    let mut bits = Vec::new();

    if file.dir.is_none() {
        if let Some(ref parent) = file.path.parent() {
            if parent.components().count() > 0 {
                bits.push(Style::default().paint(parent.to_string_lossy().to_string()));
                bits.push(Style::default().paint("/"));
            }
        }
    }

    bits.push(file_colour(colours, &file).paint(file.name.clone()));

    if links && file.is_link() {
        match file.link_target() {
            Ok(target) => {
                bits.push(Style::default().paint(" "));
                bits.push(colours.punctuation.paint("->"));
                bits.push(Style::default().paint(" "));

                if let Some(ref parent) = target.path.parent() {
                    let coconut = parent.components().count();
                    if coconut != 0 {
                        if !(coconut == 1 && parent.has_root()) {
                            bits.push(colours.symlink_path.paint(parent.to_string_lossy().to_string()));
                        }
                        bits.push(colours.symlink_path.paint("/"));
                    }
                }

                bits.push(file_colour(colours, &target).paint(target.name));
            },

            Err(filename) => {
                bits.push(Style::default().paint(" "));
                bits.push(colours.broken_arrow.paint("->"));
                bits.push(Style::default().paint(" "));
                bits.push(colours.broken_filename.paint(filename));
            },
        }
    }

    bits.into()
}

pub fn file_colour(colours: &Colours, file: &File) -> Style {
    match file {
        f if f.is_directory()        => colours.filetypes.directory,
        f if f.is_executable_file()  => colours.filetypes.executable,
        f if f.is_link()             => colours.filetypes.symlink,
        f if !f.is_file()            => colours.filetypes.special,
        f if f.is_immediate()        => colours.filetypes.immediate,
        f if f.is_image()            => colours.filetypes.image,
        f if f.is_video()            => colours.filetypes.video,
        f if f.is_music()            => colours.filetypes.music,
        f if f.is_lossless()         => colours.filetypes.lossless,
        f if f.is_crypto()           => colours.filetypes.crypto,
        f if f.is_document()         => colours.filetypes.document,
        f if f.is_compressed()       => colours.filetypes.compressed,
        f if f.is_temp()             => colours.filetypes.temp,
        f if f.is_compiled()         => colours.filetypes.compiled,
        _                            => colours.filetypes.normal,
    }
}
