use ansi_term::{ANSIString, Style};

use fs::{File, FileTarget};

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


pub fn filename(file: &File, colours: &Colours, links: bool, classify: bool) -> TextCellContents {
    let mut bits = Vec::new();

    // TODO: This long function could do with some splitting up.

    if file.dir.is_none() {
        if let Some(parent) = file.path.parent() {
            let coconut = parent.components().count();

            if coconut == 1 && parent.has_root() {
                bits.push(colours.symlink_path.paint("/"));
            }
            else if coconut >= 1 {
                bits.push(colours.symlink_path.paint(parent.to_string_lossy().to_string()));
                bits.push(colours.symlink_path.paint("/"));
            }
        }
    }

    if !file.name.is_empty() {
        for bit in coloured_file_name(file, colours) {
            bits.push(bit);
        }
    }

    if links && file.is_link() {
        match file.link_target() {
            FileTarget::Ok(target) => {
                bits.push(Style::default().paint(" "));
                bits.push(colours.punctuation.paint("->"));
                bits.push(Style::default().paint(" "));

                if let Some(parent) = target.path.parent() {
                    let coconut = parent.components().count();

                    if coconut == 1 && parent.has_root() {
                        bits.push(colours.symlink_path.paint("/"));
                    }
                    else if coconut >= 1 {
                        bits.push(colours.symlink_path.paint(parent.to_string_lossy().to_string()));
                        bits.push(colours.symlink_path.paint("/"));
                    }
                }

                if !target.name.is_empty() {
                    bits.push(file_colour(colours, &target).paint(target.name));
                }
            },

            FileTarget::Broken(broken_path) => {
                bits.push(Style::default().paint(" "));
                bits.push(colours.broken_arrow.paint("->"));
                bits.push(Style::default().paint(" "));
                bits.push(colours.broken_filename.paint(broken_path.display().to_string()));
            },

            FileTarget::Err(_) => {
                // Do nothing -- the error gets displayed on the next line
            }
        }
    } else if classify {
        if file.is_executable_file() {
            bits.push(Style::default().paint("*"));
        } else if file.is_directory() {
            bits.push(Style::default().paint("/"));
        } else if file.is_pipe() {
            bits.push(Style::default().paint("|"));
        } else if file.is_link() {
            bits.push(Style::default().paint("@"));
        } else if file.is_socket() {
            bits.push(Style::default().paint("="));
        }
    }

    bits.into()
}

/// Returns at least one ANSI-highlighted string representing this file’s
/// name using the given set of colours.
///
/// Ordinarily, this will be just one string: the file’s complete name,
/// coloured according to its file type. If the name contains control
/// characters such as newlines or escapes, though, we can’t just print them
/// to the screen directly, because then there’ll be newlines in weird places.
///
/// So in that situation, those characters will be escaped and highlighted in
/// a different colour.
fn coloured_file_name<'a>(file: &File, colours: &Colours) -> Vec<ANSIString<'a>> {
    let colour = file_colour(colours, file);
    let mut bits = Vec::new();

    if file.name.chars().all(|c| c >= 0x20 as char) {
        bits.push(colour.paint(file.name.clone()));
    }
    else {
        for c in file.name.chars() {
            // The `escape_default` method on `char` is *almost* what we want here, but
            // it still escapes non-ASCII UTF-8 characters, which are still printable.

            if c >= 0x20 as char {
                // TODO: This allocates way too much,
                // hence the `all` check above.
                let mut s = String::new();
                s.push(c);
                bits.push(colour.paint(s));
            } else {
                let s = c.escape_default().collect::<String>();
                bits.push(colours.broken_arrow.paint(s));
            }
        }
    }

    bits
}

pub fn file_colour(colours: &Colours, file: &File) -> Style {
    match file {
        f if f.is_directory()        => colours.filetypes.directory,
        f if f.is_executable_file()  => colours.filetypes.executable,
        f if f.is_link()             => colours.filetypes.symlink,
        f if f.is_pipe()             => colours.filetypes.pipe,
        f if f.is_char_device()
           | f.is_block_device()     => colours.filetypes.device,
        f if f.is_socket()           => colours.filetypes.socket,
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
