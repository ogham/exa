use users::{Users, Groups};

use fs::fields as f;
use output::colours::Colours;
use output::cell::TextCell;


impl f::Group {
    pub fn render<U: Users+Groups>(&self, colours: &Colours, users: &U) -> TextCell {
        use users::os::unix::GroupExt;

        let mut style = colours.users.group_not_yours;

        let group = match users.get_group_by_gid(self.0) {
            Some(g) => (*g).clone(),
            None    => return TextCell::paint(style, self.0.to_string()),
        };

        let current_uid = users.get_current_uid();
        if let Some(current_user) = users.get_user_by_uid(current_uid) {
            if current_user.primary_group_id() == group.gid()
            || group.members().contains(&current_user.name().to_owned()) {
                style = colours.users.group_yours;
            }
        }

        TextCell::paint(style, group.name().to_owned())
    }
}


#[cfg(test)]
#[allow(unused_results)]
pub mod test {
    use fs::fields as f;
    use output::cell::TextCell;
    use output::colours::Colours;

    use users::{User, Group};
    use users::mock::MockUsers;
    use users::os::unix::GroupExt;
    use ansi_term::Colour::*;


    #[test]
    fn named() {
        let mut colours = Colours::default();
        colours.users.group_not_yours = Fixed(101).normal();

        let mut users = MockUsers::with_current_uid(1000);
        users.add_group(Group::new(100, "folk"));

        let group = f::Group(100);
        let expected = TextCell::paint_str(Fixed(101).normal(), "folk");
        assert_eq!(expected, group.render(&colours, &users))
    }

    #[test]
    fn unnamed() {
        let mut colours = Colours::default();
        colours.users.group_not_yours = Fixed(87).normal();

        let users = MockUsers::with_current_uid(1000);

        let group = f::Group(100);
        let expected = TextCell::paint_str(Fixed(87).normal(), "100");
        assert_eq!(expected, group.render(&colours, &users));
    }

    #[test]
    fn primary() {
        let mut colours = Colours::default();
        colours.users.group_yours = Fixed(64).normal();

        let mut users = MockUsers::with_current_uid(2);
        users.add_user(User::new(2, "eve", 100));
        users.add_group(Group::new(100, "folk"));

        let group = f::Group(100);
        let expected = TextCell::paint_str(Fixed(64).normal(), "folk");
        assert_eq!(expected, group.render(&colours, &users))
    }

    #[test]
    fn secondary() {
        let mut colours = Colours::default();
        colours.users.group_yours = Fixed(31).normal();

        let mut users = MockUsers::with_current_uid(2);
        users.add_user(User::new(2, "eve", 666));

        let test_group = Group::new(100, "folk").add_member("eve");
        users.add_group(test_group);

        let group = f::Group(100);
        let expected = TextCell::paint_str(Fixed(31).normal(), "folk");
        assert_eq!(expected, group.render(&colours, &users))
    }

    #[test]
    fn overflow() {
        let mut colours = Colours::default();
        colours.users.group_not_yours = Blue.underline();

        let group = f::Group(2_147_483_648);
        let expected = TextCell::paint_str(Blue.underline(), "2147483648");
        assert_eq!(expected, group.render(&colours, &MockUsers::with_current_uid(0)));
    }
}
