use fs::fields as f;
use output::colours::Colours;
use output::cell::{TextCell, DisplayWidth};
use ansi_term::{ANSIString, Style};


impl f::PermissionsPlus {
    pub fn render(&self, colours: &Colours) -> TextCell {
        let mut chars = vec![ self.file_type.render(colours) ];
        chars.extend(self.permissions.render(colours, self.file_type.is_regular_file()));

        if self.xattrs {
           chars.push(colours.perms.attribute.paint("@"));
        }

        // As these are all ASCII characters, we can guarantee that they’re
        // all going to be one character wide, and don’t need to compute the
        // cell’s display width.
        TextCell {
            width:    DisplayWidth::from(chars.len()),
            contents: chars.into(),
        }
    }
}

impl f::Permissions {
    pub fn render(&self, colours: &Colours, is_regular_file: bool) -> Vec<ANSIString<'static>> {
        let bit = |bit, chr: &'static str, style: Style| {
            if bit { style.paint(chr) } else { colours.punctuation.paint("-") }
        };

        vec![
            bit(self.user_read,     "r", colours.perms.user_read),
            bit(self.user_write,    "w", colours.perms.user_write),
            self.user_execute_bit(colours, is_regular_file),
            bit(self.group_read,    "r", colours.perms.group_read),
            bit(self.group_write,   "w", colours.perms.group_write),
            self.group_execute_bit(colours),
            bit(self.other_read,    "r", colours.perms.other_read),
            bit(self.other_write,   "w", colours.perms.other_write),
            self.other_execute_bit(colours)
        ]
    }

    fn user_execute_bit(&self, colours: &Colours, is_regular_file: bool) -> ANSIString<'static> {
        match (self.user_execute, self.setuid, is_regular_file) {
            (false, false, _)      => colours.punctuation.paint("-"),
            (true,  false, false)  => colours.perms.user_execute_other.paint("x"),
            (true,  false, true)   => colours.perms.user_execute_file.paint("x"),
            (false, true, _)       => colours.perms.special_other.paint("S"),
            (true,  true, false)   => colours.perms.special_other.paint("s"),
            (true,  true, true)    => colours.perms.special_user_file.paint("s"),
        }
    }

    fn group_execute_bit(&self, colours: &Colours) -> ANSIString<'static> {
        match (self.group_execute, self.setgid) {
            (false, false)  => colours.punctuation.paint("-"),
            (true,  false)  => colours.perms.group_execute.paint("x"),
            (false, true)   => colours.perms.special_other.paint("S"),
            (true,  true)   => colours.perms.special_other.paint("s"),
        }
    }

    fn other_execute_bit(&self, colours: &Colours) -> ANSIString<'static> {
        match (self.other_execute, self.sticky) {
            (false, false)  => colours.punctuation.paint("-"),
            (true,  false)  => colours.perms.other_execute.paint("x"),
            (false, true)   => colours.perms.special_other.paint("T"),
            (true,  true)   => colours.perms.special_other.paint("t"),
        }
    }
}

impl f::Type {
    pub fn render(&self, colours: &Colours) -> ANSIString<'static> {
        match *self {
            f::Type::File        => colours.filetypes.normal.paint("."),
            f::Type::Directory   => colours.filetypes.directory.paint("d"),
            f::Type::Pipe        => colours.filetypes.pipe.paint("|"),
            f::Type::Link        => colours.filetypes.symlink.paint("l"),
            f::Type::CharDevice  => colours.filetypes.device.paint("c"),
            f::Type::BlockDevice => colours.filetypes.device.paint("b"),
            f::Type::Socket      => colours.filetypes.socket.paint("s"),
            f::Type::Special     => colours.filetypes.special.paint("?"),
        }
    }
}



#[cfg(test)]
#[allow(unused_results)]
pub mod test {
    use output::details::Details;
    use output::cell::TextCellContents;
    use fs::fields as f;

    use ansi_term::Colour::*;


    #[test]
    fn negate() {
        let mut details = Details::default();
        details.colours.punctuation = Fixed(11).normal();

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

        assert_eq!(expected, bits.render(&details.colours, false).into())
    }


    #[test]
    fn affirm() {
        let mut details = Details::default();
        details.colours.perms.user_read    = Fixed(101).normal();
        details.colours.perms.user_write   = Fixed(102).normal();
        details.colours.perms.user_execute_file = Fixed(103).normal();

        details.colours.perms.group_read    = Fixed(104).normal();
        details.colours.perms.group_write   = Fixed(105).normal();
        details.colours.perms.group_execute = Fixed(106).normal();

        details.colours.perms.other_read    = Fixed(107).normal();
        details.colours.perms.other_write   = Fixed(108).normal();
        details.colours.perms.other_execute = Fixed(109).normal();

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

        assert_eq!(expected, bits.render(&details.colours, true).into())
    }


    #[test]
    fn specials() {
        let mut details = Details::default();
        details.colours.punctuation = Fixed(11).normal();
        details.colours.perms.special_user_file = Fixed(77).normal();
        details.colours.perms.special_other = Fixed(88).normal();

        let bits = f::Permissions {
            user_read:  false,  user_write:  false,  user_execute:  true,  setuid: true,
            group_read: false,  group_write: false,  group_execute: true,  setgid: true,
            other_read: false,  other_write: false,  other_execute: true,  sticky: true,
        };

        let expected = TextCellContents::from(vec![
            Fixed(11).paint("-"),  Fixed(11).paint("-"),  Fixed(77).paint("s"),
            Fixed(11).paint("-"),  Fixed(11).paint("-"),  Fixed(88).paint("s"),
            Fixed(11).paint("-"),  Fixed(11).paint("-"),  Fixed(88).paint("t"),
        ]);

        assert_eq!(expected, bits.render(&details.colours, true).into())
    }


    #[test]
    fn extra_specials() {
        let mut details = Details::default();
        details.colours.punctuation = Fixed(11).normal();
        details.colours.perms.special_other = Fixed(88).normal();

        let bits = f::Permissions {
            user_read:  false,  user_write:  false,  user_execute:  false,  setuid: true,
            group_read: false,  group_write: false,  group_execute: false,  setgid: true,
            other_read: false,  other_write: false,  other_execute: false,  sticky: true,
        };

        let expected = TextCellContents::from(vec![
            Fixed(11).paint("-"),  Fixed(11).paint("-"),  Fixed(88).paint("S"),
            Fixed(11).paint("-"),  Fixed(11).paint("-"),  Fixed(88).paint("S"),
            Fixed(11).paint("-"),  Fixed(11).paint("-"),  Fixed(88).paint("T"),
        ]);

        assert_eq!(expected, bits.render(&details.colours, true).into())
    }
}
