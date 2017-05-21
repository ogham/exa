use fs::fields as f;
use output::colours::Colours;
use output::cell::{TextCell, DisplayWidth};
use ansi_term::{ANSIString, Style};


impl f::PermissionsPlus {
    pub fn render(&self, colours: &Colours) -> TextCell {
        let x_colour = if self.file_type.is_regular_file() { colours.perms.user_execute_file }
                                                      else { colours.perms.user_execute_other };

        let mut chars = vec![ self.file_type.render(colours) ];
        chars.extend(self.permissions.render(colours, x_colour));

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
    pub fn render(&self, colours: &Colours, x_colour: Style) -> Vec<ANSIString<'static>> {
        let bit = |bit, chr: &'static str, style: Style| {
            if bit { style.paint(chr) } else { colours.punctuation.paint("-") }
        };

        vec![
            bit(self.user_read,     "r", colours.perms.user_read),
            bit(self.user_write,    "w", colours.perms.user_write),
            bit(self.user_execute,  "x", x_colour),
            bit(self.group_read,    "r", colours.perms.group_read),
            bit(self.group_write,   "w", colours.perms.group_write),
            bit(self.group_execute, "x", colours.perms.group_execute),
            bit(self.other_read,    "r", colours.perms.other_read),
            bit(self.other_write,   "w", colours.perms.other_write),
            bit(self.other_execute, "x", colours.perms.other_execute),
        ]
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
        details.colours.punctuation = Fixed(44).normal();

        let bits = f::Permissions {
            user_read:  false,  user_write:  false,  user_execute:  false,
            group_read: false,  group_write: false,  group_execute: false,
            other_read: false,  other_write: false,  other_execute: false,
        };

        let expected = TextCellContents::from(vec![
            Fixed(44).paint("-"),  Fixed(44).paint("-"),  Fixed(44).paint("-"),
            Fixed(44).paint("-"),  Fixed(44).paint("-"),  Fixed(44).paint("-"),
            Fixed(44).paint("-"),  Fixed(44).paint("-"),  Fixed(44).paint("-"),
        ]);

        assert_eq!(expected, bits.render(&details.colours, Fixed(66).normal()).into())
    }


    #[test]
    fn affirm() {
        let mut details = Details::default();
        details.colours.perms.user_read    = Fixed(101).normal();
        details.colours.perms.user_write   = Fixed(102).normal();

        details.colours.perms.group_read    = Fixed(104).normal();
        details.colours.perms.group_write   = Fixed(105).normal();
        details.colours.perms.group_execute = Fixed(106).normal();

        details.colours.perms.other_read    = Fixed(107).normal();
        details.colours.perms.other_write   = Fixed(108).normal();
        details.colours.perms.other_execute = Fixed(109).normal();

        let bits = f::Permissions {
            user_read:  true,  user_write:  true,  user_execute:  true,
            group_read: true,  group_write: true,  group_execute: true,
            other_read: true,  other_write: true,  other_execute: true,
        };

        let expected = TextCellContents::from(vec![
            Fixed(101).paint("r"),  Fixed(102).paint("w"),  Fixed(103).paint("x"),
            Fixed(104).paint("r"),  Fixed(105).paint("w"),  Fixed(106).paint("x"),
            Fixed(107).paint("r"),  Fixed(108).paint("w"),  Fixed(109).paint("x"),
        ]);

        assert_eq!(expected, bits.render(&details.colours, Fixed(103).normal()).into())
    }
}
