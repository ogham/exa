use ansi_term::{ANSIString, Style};

use crate::fs::fields as f;
use crate::output::cell::{TextCell, DisplayWidth};
use crate::output::render::FiletypeColours;


impl f::PermissionsPlus {
    #[cfg(unix)]
    pub fn render<C: Colours+FiletypeColours>(&self, colours: &C) -> TextCell {
        let mut chars = vec![ self.file_type.render(colours) ];
        chars.extend(self.permissions.render(colours, self.file_type.is_regular_file()));

        if self.xattrs {
           chars.push(colours.attribute().paint("@"));
        }

        // As these are all ASCII characters, we can guarantee that they’re
        // all going to be one character wide, and don’t need to compute the
        // cell’s display width.
        TextCell {
            width:    DisplayWidth::from(chars.len()),
            contents: chars.into(),
        }
    }

    #[cfg(windows)]
    pub fn render<C: Colours+FiletypeColours>(&self, colours: &C) -> TextCell {
        let mut chars = vec![ self.attributes.render_type(colours) ];
        chars.extend(self.attributes.render(colours));

        TextCell {
            width:    DisplayWidth::from(chars.len()),
            contents: chars.into(),
        }
    }
}


impl f::Permissions {
    pub fn render<C: Colours>(&self, colours: &C, is_regular_file: bool) -> Vec<ANSIString<'static>> {

        let bit = |bit, chr: &'static str, style: Style| {
            if bit { style.paint(chr) }
              else { colours.dash().paint("-") }
        };

        vec![
            bit(self.user_read,   "r", colours.user_read()),
            bit(self.user_write,  "w", colours.user_write()),
            self.user_execute_bit(colours, is_regular_file),
            bit(self.group_read,  "r", colours.group_read()),
            bit(self.group_write, "w", colours.group_write()),
            self.group_execute_bit(colours),
            bit(self.other_read,  "r", colours.other_read()),
            bit(self.other_write, "w", colours.other_write()),
            self.other_execute_bit(colours)
        ]
    }

    fn user_execute_bit<C: Colours>(&self, colours: &C, is_regular_file: bool) -> ANSIString<'static> {
        match (self.user_execute, self.setuid, is_regular_file) {
            (false, false, _)      => colours.dash().paint("-"),
            (true,  false, false)  => colours.user_execute_other().paint("x"),
            (true,  false, true)   => colours.user_execute_file().paint("x"),
            (false, true,  _)      => colours.special_other().paint("S"),
            (true,  true,  false)  => colours.special_other().paint("s"),
            (true,  true,  true)   => colours.special_user_file().paint("s"),
        }
    }

    fn group_execute_bit<C: Colours>(&self, colours: &C) -> ANSIString<'static> {
        match (self.group_execute, self.setgid) {
            (false, false)  => colours.dash().paint("-"),
            (true,  false)  => colours.group_execute().paint("x"),
            (false, true)   => colours.special_other().paint("S"),
            (true,  true)   => colours.special_other().paint("s"),
        }
    }

    fn other_execute_bit<C: Colours>(&self, colours: &C) -> ANSIString<'static> {
        match (self.other_execute, self.sticky) {
            (false, false)  => colours.dash().paint("-"),
            (true,  false)  => colours.other_execute().paint("x"),
            (false, true)   => colours.special_other().paint("T"),
            (true,  true)   => colours.special_other().paint("t"),
        }
    }
}

impl f::Attributes {
    pub fn render<C: Colours+FiletypeColours>(&self, colours: &C) -> Vec<ANSIString<'static>> {
        let bit = |bit, chr: &'static str, style: Style| {
            if bit { style.paint(chr) }
              else { colours.dash().paint("-") }
        };

        vec![
            bit(self.archive,   "a", colours.normal()),
            bit(self.readonly,  "r", colours.user_read()),
            bit(self.hidden,    "h", colours.special_user_file()),
            bit(self.system,    "s", colours.special_other()),
        ]
    }

    pub fn render_type<C: Colours+FiletypeColours>(&self, colours: &C) -> ANSIString<'static> {
        if self.reparse_point {
            return colours.pipe().paint("l")
        }
        else if self.directory {
            return colours.directory().paint("d")
        }
        else {
            return colours.dash().paint("-")
        }
    }
}

pub trait Colours {
    fn dash(&self) -> Style;

    fn user_read(&self) -> Style;
    fn user_write(&self) -> Style;
    fn user_execute_file(&self) -> Style;
    fn user_execute_other(&self) -> Style;

    fn group_read(&self) -> Style;
    fn group_write(&self) -> Style;
    fn group_execute(&self) -> Style;

    fn other_read(&self) -> Style;
    fn other_write(&self) -> Style;
    fn other_execute(&self) -> Style;

    fn special_user_file(&self) -> Style;
    fn special_other(&self) -> Style;

    fn attribute(&self) -> Style;
}


#[cfg(test)]
#[allow(unused_results)]
pub mod test {
    use super::Colours;
    use crate::output::cell::TextCellContents;
    use crate::fs::fields as f;

    use ansi_term::Colour::*;
    use ansi_term::Style;


    struct TestColours;

    impl Colours for TestColours {
        fn dash(&self)                -> Style { Fixed(11).normal() }
        fn user_read(&self)           -> Style { Fixed(101).normal() }
        fn user_write(&self)          -> Style { Fixed(102).normal() }
        fn user_execute_file(&self)   -> Style { Fixed(103).normal() }
        fn user_execute_other(&self)  -> Style { Fixed(113).normal() }
        fn group_read(&self)          -> Style { Fixed(104).normal() }
        fn group_write(&self)         -> Style { Fixed(105).normal() }
        fn group_execute(&self)       -> Style { Fixed(106).normal() }
        fn other_read(&self)          -> Style { Fixed(107).normal() }
        fn other_write(&self)         -> Style { Fixed(108).normal() }
        fn other_execute(&self)       -> Style { Fixed(109).normal() }
        fn special_user_file(&self)   -> Style { Fixed(110).normal() }
        fn special_other(&self)       -> Style { Fixed(111).normal() }
        fn attribute(&self)           -> Style { Fixed(112).normal() }
    }


    #[test]
    fn negate() {
        let bits = f::Permissions {
            user_read:  false,  user_write:  false,  user_execute:  false,  setuid: false,
            group_read: false,  group_write: false,  group_execute: false,  setgid: false,
            other_read: false,  other_write: false,  other_execute: false,  sticky: false,
        };

        let expected = TextCellContents::from(vec![
            Fixed(11).paint("-"),  Fixed(11).paint("-"),  Fixed(11).paint("-"),
            Fixed(11).paint("-"),  Fixed(11).paint("-"),  Fixed(11).paint("-"),
            Fixed(11).paint("-"),  Fixed(11).paint("-"),  Fixed(11).paint("-"),
        ]);

        assert_eq!(expected, bits.render(&TestColours, false).into())
    }


    #[test]
    fn affirm() {
        let bits = f::Permissions {
            user_read:  true,  user_write:  true,  user_execute:  true,  setuid: false,
            group_read: true,  group_write: true,  group_execute: true,  setgid: false,
            other_read: true,  other_write: true,  other_execute: true,  sticky: false,
        };

        let expected = TextCellContents::from(vec![
            Fixed(101).paint("r"),  Fixed(102).paint("w"),  Fixed(103).paint("x"),
            Fixed(104).paint("r"),  Fixed(105).paint("w"),  Fixed(106).paint("x"),
            Fixed(107).paint("r"),  Fixed(108).paint("w"),  Fixed(109).paint("x"),
        ]);

        assert_eq!(expected, bits.render(&TestColours, true).into())
    }


    #[test]
    fn specials() {
        let bits = f::Permissions {
            user_read:  false,  user_write:  false,  user_execute:  true,  setuid: true,
            group_read: false,  group_write: false,  group_execute: true,  setgid: true,
            other_read: false,  other_write: false,  other_execute: true,  sticky: true,
        };

        let expected = TextCellContents::from(vec![
            Fixed(11).paint("-"),  Fixed(11).paint("-"),  Fixed(110).paint("s"),
            Fixed(11).paint("-"),  Fixed(11).paint("-"),  Fixed(111).paint("s"),
            Fixed(11).paint("-"),  Fixed(11).paint("-"),  Fixed(111).paint("t"),
        ]);

        assert_eq!(expected, bits.render(&TestColours, true).into())
    }


    #[test]
    fn extra_specials() {
        let bits = f::Permissions {
            user_read:  false,  user_write:  false,  user_execute:  false,  setuid: true,
            group_read: false,  group_write: false,  group_execute: false,  setgid: true,
            other_read: false,  other_write: false,  other_execute: false,  sticky: true,
        };

        let expected = TextCellContents::from(vec![
            Fixed(11).paint("-"),  Fixed(11).paint("-"),  Fixed(111).paint("S"),
            Fixed(11).paint("-"),  Fixed(11).paint("-"),  Fixed(111).paint("S"),
            Fixed(11).paint("-"),  Fixed(11).paint("-"),  Fixed(111).paint("T"),
        ]);

        assert_eq!(expected, bits.render(&TestColours, true).into())
    }
}
