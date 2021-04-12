use std::fmt::Debug;
use std::path::Path;

use ansi_term::{ANSIString, Style};

use crate::fs::{File, FileTarget};
use crate::output::cell::TextCellContents;
use crate::output::escape;
use crate::output::icons::{icon_for_file, iconify_style};
use crate::output::render::FiletypeColours;


/// Basically a file name factory.
#[derive(Debug, Copy, Clone)]
pub struct Options {

    /// Whether to append file class characters to file names.
    pub classify: Classify,

    /// Whether to prepend icon characters before file names.
    pub show_icons: ShowIcons,
}

impl Options {

    /// Create a new `FileName` that prints the given file’s name, painting it
    /// with the remaining arguments.
    pub fn for_file<'a, 'dir, C>(self, file: &'a File<'dir>, colours: &'a C) -> FileName<'a, 'dir, C> {
        FileName {
            file,
            colours,
            link_style: LinkStyle::JustFilenames,
            options:    self,
            target:     if file.is_link() { Some(file.link_target()) }
                                     else { None }
        }
    }
}

/// When displaying a file name, there needs to be some way to handle broken
/// links, depending on how long the resulting Cell can be.
#[derive(PartialEq, Debug, Copy, Clone)]
enum LinkStyle {

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
    fn default() -> Self {
        Self::JustFilenames
    }
}


/// Whether and how to show icons.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ShowIcons {

    /// Don’t show icons at all.
    Off,

    /// Show icons next to file names, with the given number of spaces between
    /// the icon and the file name.
    On(u32),
}


/// A **file name** holds all the information necessary to display the name
/// of the given file. This is used in all of the views.
pub struct FileName<'a, 'dir, C> {

    /// A reference to the file that we’re getting the name of.
    file: &'a File<'dir>,

    /// The colours used to paint the file name and its surrounding text.
    colours: &'a C,

    /// The file that this file points to if it’s a link.
    target: Option<FileTarget<'dir>>,  // todo: remove?

    /// How to handle displaying links.
    link_style: LinkStyle,

    options: Options,
}

impl<'a, 'dir, C> FileName<'a, 'dir, C> {

    /// Sets the flag on this file name to display link targets with an
    /// arrow followed by their path.
    pub fn with_link_paths(mut self) -> Self {
        self.link_style = LinkStyle::FullLinkPaths;
        self
    }
}

impl<'a, 'dir, C: Colours> FileName<'a, 'dir, C> {

    /// Paints the name of the file using the colours, resulting in a vector
    /// of coloured cells that can be printed to the terminal.
    ///
    /// This method returns some `TextCellContents`, rather than a `TextCell`,
    /// because for the last cell in a table, it doesn’t need to have its
    /// width calculated.
    pub fn paint(&self) -> TextCellContents {
        let mut bits = Vec::new();

        if let ShowIcons::On(spaces_count) = self.options.show_icons {
            let style = iconify_style(self.style());
            let file_icon = icon_for_file(self.file).to_string();

            bits.push(style.paint(file_icon));

            match spaces_count {
                1 => bits.push(style.paint(" ")),
                2 => bits.push(style.paint("  ")),
                n => bits.push(style.paint(spaces(n))),
            }
        }

        if self.file.parent_dir.is_none() {
            if let Some(parent) = self.file.path.parent() {
                self.add_parent_bits(&mut bits, parent);
            }
        }

        if ! self.file.name.is_empty() {
        	// The “missing file” colour seems like it should be used here,
        	// but it’s not! In a grid view, where there’s no space to display
        	// link targets, the filename has to have a different style to
        	// indicate this fact. But when showing targets, we can just
        	// colour the path instead (see below), and leave the broken
        	// link’s filename as the link colour.
            for bit in self.coloured_file_name() {
                bits.push(bit);
            }
        }

        if let (LinkStyle::FullLinkPaths, Some(target)) = (self.link_style, self.target.as_ref()) {
            match target {
                FileTarget::Ok(target) => {
                    bits.push(Style::default().paint(" "));
                    bits.push(self.colours.normal_arrow().paint("->"));
                    bits.push(Style::default().paint(" "));

                    if let Some(parent) = target.path.parent() {
                        self.add_parent_bits(&mut bits, parent);
                    }

                    if ! target.name.is_empty() {
                        let target_options = Options {
                            classify: Classify::JustFilenames,
                            show_icons: ShowIcons::Off,
                        };

                        let target_name = FileName {
                            file: target,
                            colours: self.colours,
                            target: None,
                            link_style: LinkStyle::FullLinkPaths,
                            options: target_options,
                        };

                        for bit in target_name.coloured_file_name() {
                            bits.push(bit);
                        }

                        if let Classify::AddFileIndicators = self.options.classify {
                            if let Some(class) = self.classify_char(target) {
                                bits.push(Style::default().paint(class));
                            }
                        }
                    }
                }

                FileTarget::Broken(broken_path) => {
                    bits.push(Style::default().paint(" "));
                    bits.push(self.colours.broken_symlink().paint("->"));
                    bits.push(Style::default().paint(" "));

                    escape(
                        broken_path.display().to_string(),
                        &mut bits,
                        self.colours.broken_filename(),
                        self.colours.broken_control_char(),
                    );
                }

                FileTarget::Err(_) => {
                    // Do nothing — the error gets displayed on the next line
                }
            }
        }
        else if let Classify::AddFileIndicators = self.options.classify {
            if let Some(class) = self.classify_char(self.file) {
                bits.push(Style::default().paint(class));
            }
        }

        bits.into()
    }

    /// Adds the bits of the parent path to the given bits vector.
    /// The path gets its characters escaped based on the colours.
    fn add_parent_bits(&self, bits: &mut Vec<ANSIString<'_>>, parent: &Path) {
        let coconut = parent.components().count();

        if coconut == 1 && parent.has_root() {
            bits.push(self.colours.symlink_path().paint("/"));
        }
        else if coconut >= 1 {
            escape(
                parent.to_string_lossy().to_string(),
                bits,
                self.colours.symlink_path(),
                self.colours.control_char(),
            );
            bits.push(self.colours.symlink_path().paint("/"));
        }
    }

    /// The character to be displayed after a file when classifying is on, if
    /// the file’s type has one associated with it.
    fn classify_char(&self, file: &File<'_>) -> Option<&'static str> {
        if file.is_executable_file() {
            Some("*")
        }
        else if file.is_directory() {
            Some("/")
        }
        else if file.is_pipe() {
            Some("|")
        }
        else if file.is_link() {
            Some("@")
        }
        else if file.is_socket() {
            Some("=")
        }
        else {
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

        escape(
            self.file.name.clone(),
            &mut bits,
            file_style,
            self.colours.control_char(),
        );

        bits
    }

    /// Figures out which colour to paint the filename part of the output,
    /// depending on which “type” of file it appears to be — either from the
    /// class on the filesystem or from its name. (Or the broken link colour,
    /// if there’s nowhere else for that fact to be shown.)
    pub fn style(&self) -> Style {
        if let LinkStyle::JustFilenames = self.link_style {
            if let Some(ref target) = self.target {
                if target.is_broken() {
                    return self.colours.broken_symlink();
                }
            }
        }

        match self.file {
            f if f.is_directory()        => self.colours.directory(),
            f if f.is_executable_file()  => self.colours.executable_file(),
            f if f.is_link()             => self.colours.symlink(),
            f if f.is_pipe()             => self.colours.pipe(),
            f if f.is_block_device()     => self.colours.block_device(),
            f if f.is_char_device()      => self.colours.char_device(),
            f if f.is_socket()           => self.colours.socket(),
            f if ! f.is_file()           => self.colours.special(),
            _                            => self.colours.colour_file(self.file),
        }
    }
}


/// The set of colours that are needed to paint a file name.
pub trait Colours: FiletypeColours {

    /// The style to paint the path of a symlink’s target, up to but not
    /// including the file’s name.
    fn symlink_path(&self) -> Style;

    /// The style to paint the arrow between a link and its target.
    fn normal_arrow(&self) -> Style;

	/// The style to paint the filenames of broken links in views that don’t
	/// show link targets, and the style to paint the *arrow* between the link
	/// and its target in views that *do* show link targets.
    fn broken_symlink(&self) -> Style;

    /// The style to paint the entire filename of a broken link.
    fn broken_filename(&self) -> Style;

    /// The style to paint a non-displayable control character in a filename.
    fn control_char(&self) -> Style;

    /// The style to paint a non-displayable control character in a filename,
    /// when the filename is being displayed as a broken link target.
    fn broken_control_char(&self) -> Style;

    /// The style to paint a file that has its executable bit set.
    fn executable_file(&self) -> Style;

    fn colour_file(&self, file: &File<'_>) -> Style;
}


/// Generate a string made of `n` spaces.
fn spaces(width: u32) -> String {
    (0 .. width).into_iter().map(|_| ' ').collect()
}
