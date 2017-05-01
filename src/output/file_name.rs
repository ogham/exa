use std::path::Path;

use ansi_term::{ANSIString, Style};

use fs::{File, FileTarget};
use output::Colours;
use output::escape;
use output::cell::TextCellContents;


pub struct FileName<'a, 'dir: 'a> {
    file:    &'a File<'dir>,
    colours: &'a Colours,
}

impl<'a, 'dir> FileName<'a, 'dir> {
    pub fn new(file: &'a File<'dir>, colours: &'a Colours) -> FileName<'a, 'dir> {
        FileName {
            file: file,
            colours: colours,
        }
    }

    pub fn file_name(&self, links: bool, classify: bool) -> TextCellContents {
        let mut bits = Vec::new();

        if self.file.dir.is_none() {
            if let Some(parent) = self.file.path.parent() {
                self.add_parent_bits(&mut bits, parent);
            }
        }

        if !self.file.name.is_empty() {
            for bit in self.coloured_file_name() {
                bits.push(bit);
            }
        }

        if links && self.file.is_link() {
            match self.file.link_target() {
                FileTarget::Ok(target) => {
                    bits.push(Style::default().paint(" "));
                    bits.push(self.colours.punctuation.paint("->"));
                    bits.push(Style::default().paint(" "));

                    if let Some(parent) = target.path.parent() {
                        self.add_parent_bits(&mut bits, parent);
                    }

                    if !target.name.is_empty() {
                        let target = FileName::new(&target, self.colours);
                        for bit in target.coloured_file_name() {
                            bits.push(bit);
                        }
                    }
                },

                FileTarget::Broken(broken_path) => {
                    bits.push(Style::default().paint(" "));
                    bits.push(self.colours.broken_arrow.paint("->"));
                    bits.push(Style::default().paint(" "));
                    bits.push(self.colours.broken_filename.paint(broken_path.display().to_string()));
                },

                FileTarget::Err(_) => {
                    // Do nothing -- the error gets displayed on the next line
                }
            }
        }
        else if classify {
            if let Some(class) = self.classify_char() {
                bits.push(Style::default().paint(class));
            }
        }

        bits.into()
    }

    /// Adds the bits of the parent path to the given bits vector.
    /// The path gets its characters escaped based on the colours.
    fn add_parent_bits(&self, bits: &mut Vec<ANSIString>, parent: &Path) {
        let coconut = parent.components().count();

        if coconut == 1 && parent.has_root() {
            bits.push(self.colours.symlink_path.paint("/"));
        }
        else if coconut >= 1 {
            escape(parent.to_string_lossy().to_string(), bits, self.colours.symlink_path, self.colours.control_char);
            bits.push(self.colours.symlink_path.paint("/"));
        }
    }

    /// The character to be displayed after a file when classifying is on, if
    /// the file’s type has one associated with it.
    fn classify_char(&self) -> Option<&'static str> {
        if self.file.is_executable_file() {
            Some("*")
        } else if self.file.is_directory() {
            Some("/")
        } else if self.file.is_pipe() {
            Some("|")
        } else if self.file.is_link() {
            Some("@")
        } else if self.file.is_socket() {
            Some("=")
        } else {
            None
        }
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
    fn coloured_file_name<'unused>(&self) -> Vec<ANSIString<'unused>> {
        let file_style = self.style();
        let mut bits = Vec::new();
        escape(self.file.name.clone(), &mut bits, file_style, self.colours.control_char);
        bits
    }

    pub fn style(&self) -> Style {
        match self.file {
            f if f.is_directory()        => self.colours.filetypes.directory,
            f if f.is_executable_file()  => self.colours.filetypes.executable,
            f if f.is_link()             => self.colours.filetypes.symlink,
            f if f.is_pipe()             => self.colours.filetypes.pipe,
            f if f.is_char_device()
               | f.is_block_device()     => self.colours.filetypes.device,
            f if f.is_socket()           => self.colours.filetypes.socket,
            f if !f.is_file()            => self.colours.filetypes.special,
            f if f.is_immediate()        => self.colours.filetypes.immediate,
            f if f.is_image()            => self.colours.filetypes.image,
            f if f.is_video()            => self.colours.filetypes.video,
            f if f.is_music()            => self.colours.filetypes.music,
            f if f.is_lossless()         => self.colours.filetypes.lossless,
            f if f.is_crypto()           => self.colours.filetypes.crypto,
            f if f.is_document()         => self.colours.filetypes.document,
            f if f.is_compressed()       => self.colours.filetypes.compressed,
            f if f.is_temp()             => self.colours.filetypes.temp,
            f if f.is_compiled()         => self.colours.filetypes.compiled,
            _                            => self.colours.filetypes.normal,
        }
    }
}
