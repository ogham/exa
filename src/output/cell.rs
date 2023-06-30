//! The `TextCell` type for the details and lines views.

use std::iter::Sum;
use std::ops::{Add, Deref, DerefMut};

use ansi_term::{Style, ANSIString, ANSIStrings};
use unicode_width::UnicodeWidthStr;


/// An individual cell that holds text in a table, used in the details and
/// lines views to store ANSI-terminal-formatted data before it is printed.
///
/// A text cell is made up of zero or more strings coupled with the
/// pre-computed length of all the strings combined. When constructing details
/// or grid-details tables, the length will have to be queried multiple times,
/// so it makes sense to cache it.
///
/// (This used to be called `Cell`, but was renamed because there’s a Rust
/// type by that name too.)
#[derive(PartialEq, Debug, Clone, Default)]
pub struct TextCell {

    /// The contents of this cell, as a vector of ANSI-styled strings.
    pub contents: TextCellContents,

    /// The Unicode “display width” of this cell.
    pub width: DisplayWidth,
}

impl Deref for TextCell {
    type Target = TextCellContents;

    fn deref(&self) -> &Self::Target {
        &self.contents
    }
}

impl TextCell {

    /// Creates a new text cell that holds the given text in the given style,
    /// computing the Unicode width of the text.
    pub fn paint(style: Style, text: String) -> Self {
        let width = DisplayWidth::from(&*text);

        Self {
            contents: vec![ style.paint(text) ].into(),
            width,
        }
    }

    /// Creates a new text cell that holds the given text in the given style,
    /// computing the Unicode width of the text. (This could be merged with
    /// `paint`, but.)
    pub fn paint_str(style: Style, text: &'static str) -> Self {
        let width = DisplayWidth::from(text);

        Self {
            contents: vec![ style.paint(text) ].into(),
            width,
        }
    }

    /// Creates a new “blank” text cell that contains a single hyphen in the
    /// given style, which should be the “punctuation” style from a `Colours`
    /// value.
    ///
    /// This is used in place of empty table cells, as it is easier to read
    /// tabular data when there is *something* in each cell.
    pub fn blank(style: Style) -> Self {
        Self {
            contents: vec![ style.paint("-") ].into(),
            width:    DisplayWidth::from(1),
        }
    }

    /// Adds the given number of unstyled spaces after this cell.
    ///
    /// This method allocates a `String` to hold the spaces.
    pub fn add_spaces(&mut self, count: usize) {
        (*self.width) += count;

        let spaces: String = " ".repeat(count);
        self.contents.0.push(Style::default().paint(spaces));
    }

    /// Adds the contents of another `ANSIString` to the end of this cell.
    pub fn push(&mut self, string: ANSIString<'static>, extra_width: usize) {
        self.contents.0.push(string);
        (*self.width) += extra_width;
    }

    /// Adds all the contents of another `TextCell` to the end of this cell.
    pub fn append(&mut self, other: Self) {
        (*self.width) += *other.width;
        self.contents.0.extend(other.contents.0);
    }
}


// I’d like to eventually abstract cells so that instead of *every* cell
// storing a vector, only variable-length cells would, and individual cells
// would just store an array of a fixed length (which would usually be just 1
// or 2), which wouldn’t require a heap allocation.
//
// For examples, look at the `render_*` methods in the `Table` object in the
// details view:
//
// - `render_blocks`, `inode`, and `links` will always return a
//   one-string-long TextCell;
// - `render_size` will return one or two strings in a TextCell, depending on
//   the size and whether one is present;
// - `render_permissions` will return ten or eleven strings;
// - `filename` and `symlink_filename` in the output module root return six or
//   five strings.
//
// In none of these cases are we dealing with a *truly variable* number of
// strings: it is only when the strings are concatenated together do we need a
// growable, heap-allocated buffer.
//
// So it would be nice to abstract the `TextCell` type so instead of a `Vec`,
// it can use anything of type `T: IntoIterator<Item=ANSIString<’static>>`.
// This would allow us to still hold all the data, but allocate less.
//
// But exa still has bugs and I need to fix those first :(


/// The contents of a text cell, as a vector of ANSI-styled strings.
///
/// It’s possible to use this type directly in the case where you want a
/// `TextCell` but aren’t concerned with tracking its width, because it occurs
/// in the final cell of a table or grid and there’s no point padding it. This
/// happens when dealing with file names.
#[derive(PartialEq, Debug, Clone, Default)]
pub struct TextCellContents(Vec<ANSIString<'static>>);

impl From<Vec<ANSIString<'static>>> for TextCellContents {
    fn from(strings: Vec<ANSIString<'static>>) -> Self {
        Self(strings)
    }
}

impl Deref for TextCellContents {
    type Target = [ANSIString<'static>];

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

// No DerefMut implementation here — it would be publicly accessible, and as
// the contents only get changed in this module, the mutators in the struct
// above can just access the value directly.

impl TextCellContents {

    /// Produces an `ANSIStrings` value that can be used to print the styled
    /// values of this cell as an ANSI-terminal-formatted string.
    pub fn strings(&self) -> ANSIStrings<'_> {
        ANSIStrings(&self.0)
    }

    /// Calculates the width that a cell with these contents would take up, by
    /// counting the number of characters in each unformatted ANSI string.
    pub fn width(&self) -> DisplayWidth {
        self.0.iter()
            .map(|anstr| DisplayWidth::from(&**anstr))
            .sum()
    }

    /// Promotes these contents to a full cell containing them alongside
    /// their calculated width.
    pub fn promote(self) -> TextCell {
        TextCell {
            width: self.width(),
            contents: self,
        }
    }
}


/// The Unicode “display width” of a string.
///
/// This is related to the number of *graphemes* of a string, rather than the
/// number of *characters*, or *bytes*: although most characters are one
/// column wide, a few can be two columns wide, and this is important to note
/// when calculating widths for displaying tables in a terminal.
///
/// This type is used to ensure that the width, rather than the length, is
/// used when constructing a `TextCell` — it’s too easy to write something
/// like `file_name.len()` and assume it will work!
///
/// It has `From` impls that convert an input string or fixed with to values
/// of this type, and will `Deref` to the contained `usize` value.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Default)]
pub struct DisplayWidth(usize);

impl<'a> From<&'a str> for DisplayWidth {
    fn from(input: &'a str) -> Self {
        Self(UnicodeWidthStr::width(input))
    }
}

impl From<usize> for DisplayWidth {
    fn from(width: usize) -> Self {
        Self(width)
    }
}

impl Deref for DisplayWidth {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DisplayWidth {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Add for DisplayWidth {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<usize> for DisplayWidth {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Sum for DisplayWidth {
    fn sum<I>(iter: I) -> Self
    where I: Iterator<Item = Self>
    {
        iter.fold(Self(0), Add::add)
    }
}


#[cfg(test)]
mod width_unit_test {
    use super::DisplayWidth;

    #[test]
    fn empty_string() {
        let cell = DisplayWidth::from("");
        assert_eq!(*cell, 0);
    }

    #[test]
    fn test_string() {
        let cell = DisplayWidth::from("Diss Playwidth");
        assert_eq!(*cell, 14);
    }

    #[test]
    fn addition() {
        let cell_one = DisplayWidth::from("/usr/bin/");
        let cell_two = DisplayWidth::from("drinking");
        assert_eq!(*(cell_one + cell_two), 17);
    }

    #[test]
    fn addition_usize() {
        let cell = DisplayWidth::from("/usr/bin/");
        assert_eq!(*(cell + 8), 17);
    }
}
