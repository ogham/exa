use fs::fields as f;
use output::colours::Colours;
use output::cell::{TextCell, DisplayWidth};
use ansi_term::Style;


impl f::Permissions {
    pub fn render(&self, colours: &Colours, file_type: f::Type, xattrs: bool) -> TextCell {
       let bit = |bit, chr: &'static str, style: Style| {
           if bit { style.paint(chr) } else { colours.punctuation.paint("-") }
       };

       let type_char = match file_type {
           f::Type::File        => colours.filetypes.normal.paint("."),
           f::Type::Directory   => colours.filetypes.directory.paint("d"),
           f::Type::Pipe        => colours.filetypes.pipe.paint("|"),
           f::Type::Link        => colours.filetypes.symlink.paint("l"),
           f::Type::CharDevice  => colours.filetypes.device.paint("c"),
           f::Type::BlockDevice => colours.filetypes.device.paint("b"),
           f::Type::Socket      => colours.filetypes.socket.paint("s"),
           f::Type::Special     => colours.filetypes.special.paint("?"),
       };

       let x_colour = if file_type.is_regular_file() { colours.perms.user_execute_file }
                                                else { colours.perms.user_execute_other };

       let mut chars = vec![
           type_char,
           bit(self.user_read,     "r", colours.perms.user_read),
           bit(self.user_write,    "w", colours.perms.user_write),
           bit(self.user_execute,  "x", x_colour),
           bit(self.group_read,    "r", colours.perms.group_read),
           bit(self.group_write,   "w", colours.perms.group_write),
           bit(self.group_execute, "x", colours.perms.group_execute),
           bit(self.other_read,    "r", colours.perms.other_read),
           bit(self.other_write,   "w", colours.perms.other_write),
           bit(self.other_execute, "x", colours.perms.other_execute),
       ];

       if xattrs {
           chars.push(colours.perms.attribute.paint("@"));
       }

       // As these are all ASCII characters, we can guarantee that they’re
       // all going to be one character wide, and don’t need to compute the
       // cell’s display width.
       let width = DisplayWidth::from(chars.len());

       TextCell {
           contents: chars.into(),
           width:    width,
       }
   }
}


#[cfg(test)]
#[allow(unused_results)]
pub mod test {
    use output::details::Details;

    use fs::fields as f;
    use output::cell::TextCell;

    use users::{User, Group};
    use users::mock::MockUsers;
    use users::os::unix::GroupExt;
    use ansi_term::Colour::*;


    #[test]
    fn named() {
        let mut details = Details::default();
        details.colours.users.group_not_yours = Fixed(101).normal();

        let mut users = MockUsers::with_current_uid(1000);
        users.add_group(Group::new(100, "folk"));

        let group = f::Group(100);
        let expected = TextCell::paint_str(Fixed(101).normal(), "folk");
        assert_eq!(expected, group.render(&details.colours, &users))
    }

    #[test]
    fn unnamed() {
        let mut details = Details::default();
        details.colours.users.group_not_yours = Fixed(87).normal();

        let users = MockUsers::with_current_uid(1000);

        let group = f::Group(100);
        let expected = TextCell::paint_str(Fixed(87).normal(), "100");
        assert_eq!(expected, group.render(&details.colours, &users));
    }

    #[test]
    fn primary() {
        let mut details = Details::default();
        details.colours.users.group_yours = Fixed(64).normal();

        let mut users = MockUsers::with_current_uid(2);
        users.add_user(User::new(2, "eve", 100));
        users.add_group(Group::new(100, "folk"));

        let group = f::Group(100);
        let expected = TextCell::paint_str(Fixed(64).normal(), "folk");
        assert_eq!(expected, group.render(&details.colours, &users))
    }

    #[test]
    fn secondary() {
        let mut details = Details::default();
        details.colours.users.group_yours = Fixed(31).normal();

        let mut users = MockUsers::with_current_uid(2);
        users.add_user(User::new(2, "eve", 666));

        let test_group = Group::new(100, "folk").add_member("eve");
        users.add_group(test_group);

        let group = f::Group(100);
        let expected = TextCell::paint_str(Fixed(31).normal(), "folk");
        assert_eq!(expected, group.render(&details.colours, &users))
    }

    #[test]
    fn overflow() {
        let mut details = Details::default();
        details.colours.users.group_not_yours = Blue.underline();

        let group = f::Group(2_147_483_648);
        let expected = TextCell::paint_str(Blue.underline(), "2147483648");
        assert_eq!(expected, group.render(&details.colours, &MockUsers::with_current_uid(0)));
    }
}
