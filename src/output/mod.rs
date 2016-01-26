use ansi_term::Style;

use file::File;

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

pub fn filename(file: File, colours: &Colours, links: bool) -> TextCellContents {
    if links && file.is_link() {
        symlink_filename(file, colours)
    }
    else {
        vec![
            file_colour(colours, &file).paint(file.name)
        ].into()
    }
}

fn symlink_filename(file: File, colours: &Colours) -> TextCellContents {
    match file.link_target() {
        Ok(target) => vec![
            file_colour(colours, &file).paint(file.name),
            Style::default().paint(" "),
            colours.punctuation.paint("->"),
            Style::default().paint(" "),
            colours.symlink_path.paint(target.path_prefix()),
            file_colour(colours, &target).paint(target.name)
        ].into(),

        Err(filename) => vec![
            file_colour(colours, &file).paint(file.name),
            Style::default().paint(" "),
            colours.broken_arrow.paint("->"),
            Style::default().paint(" "),
            colours.broken_filename.paint(filename),
        ].into(),
    }
}

pub fn file_colour(colours: &Colours, file: &File) -> Style {
    use filetype::FileTypes;

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