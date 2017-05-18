use std::path::Path;

use ansi_term::{ANSIString, Style};

use fs::{File, FileTarget};
use output::Colours;
use output::escape;
use output::cell::TextCellContents;


/// A **file name** holds all the information necessary to display the name
/// of the given file. This is used in all of the views.
pub struct FileName<'a, 'dir: 'a> {

    /// A reference to the file that we're getting the name of.
    file: &'a File<'dir>,

    /// The colours used to paint the file name and its surrounding text.
    colours: &'a Colours,

    /// The file that this file points to if it's a link.
    target: Option<FileTarget<'dir>>,

    /// How to handle displaying links.
    link_style: LinkStyle,

    /// Whether to append file class characters to file names.
    classify: Classify,
}


impl<'a, 'dir> FileName<'a, 'dir> {

    /// Create a new `FileName` that prints the given file’s name, painting it
    /// with the remaining arguments.
    pub fn new(file: &'a File<'dir>, link_style: LinkStyle, classify: Classify, colours: &'a Colours) -> FileName<'a, 'dir> {
        let target = if file.is_link() { Some(file.link_target()) }
                                                      else { None };
        FileName { file, colours, target, link_style, classify }
    }


    /// Paints the name of the file using the colours, resulting in a vector
    /// of coloured cells that can be printed to the terminal.
    ///
    /// This method returns some `TextCellContents`, rather than a `TextCell`,
    /// because for the last cell in a table, it doesn’t need to have its
    /// width calculated.
    pub fn paint(&self) -> TextCellContents {
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

        if let (LinkStyle::FullLinkPaths, Some(target)) = (self.link_style, self.target.as_ref()) {
            match *target {
                FileTarget::Ok(ref target) => {
                    bits.push(Style::default().paint(" "));
                    bits.push(self.colours.punctuation.paint("->"));
                    bits.push(Style::default().paint(" "));

                    if let Some(parent) = target.path.parent() {
                        self.add_parent_bits(&mut bits, parent);
                    }

                    if !target.name.is_empty() {
                        let target = FileName::new(target, LinkStyle::FullLinkPaths, Classify::JustFilenames, self.colours);
                        for bit in target.coloured_file_name() {
                            bits.push(bit);
                        }
                    }
                },

                FileTarget::Broken(ref broken_path) => {
                    bits.push(Style::default().paint(" "));
                    bits.push(self.colours.broken_arrow.paint("->"));
                    bits.push(Style::default().paint(" "));
                    escape(broken_path.display().to_string(), &mut bits, self.colours.broken_filename, self.colours.control_char.underline());
                },

                FileTarget::Err(_) => {
                    // Do nothing -- the error gets displayed on the next line
                },
            }
        }
        else if let Classify::AddFileIndicators = self.classify {
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


    /// Figures out which colour to paint the filename part of the output,
    /// depending on which “type” of file it appears to be -- either from the
    /// class on the filesystem or from its name.
    pub fn style(&self) -> Style {

        // Override the style with the “broken link” style when this file is
        // a link that we can’t follow for whatever reason. This is used when
        // there’s no other place to show that the link doesn’t work.
        if let LinkStyle::JustFilenames = self.link_style {
            if let Some(ref target) = self.target {
                if target.is_broken() {
                    return self.colours.broken_arrow;
                }
            }
        }

        // Otherwise, just apply a bunch of rules in order. For example,
        // executable image files should be executable rather than images.
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


/// When displaying a file name, there needs to be some way to handle broken
/// links, depending on how long the resulting Cell can be.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum LinkStyle {

    /// Just display the file names, but colour them differently if they’re
    /// a broken link or can’t be followed.
    JustFilenames,

    /// Display all files in their usual style, but follow each link with an
    /// arrow pointing to their path, colouring the path differently if it’s
    /// a broken link, and doing nothing if it can’t be followed.
    FullLinkPaths,
}


/// Whether to append file class characters to the file names.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Classify {

    /// Just display the file names, without any characters.
    JustFilenames,

    /// Add a character after the file name depending on what class of file
    /// it is.
    AddFileIndicators,
}

impl Default for Classify {
    fn default() -> Classify {
        Classify::JustFilenames
    }
}
