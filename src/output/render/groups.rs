use ansi_term::Style;
use users::{Groups, Users};

use fs::fields as f;
use output::cell::TextCell;

impl f::Group {
    pub fn render<C: Colours, U: Users + Groups>(&self, colours: &C, users: &U) -> TextCell {
        use users::os::unix::GroupExt;

        let mut style = colours.not_yours();

        let group = match users.get_group_by_gid(self.0) {
            Some(g) => (*g).clone(),
            None => return TextCell::paint(style, self.0.to_string()),
        };

        let current_uid = users.get_current_uid();
        if let Some(current_user) = users.get_user_by_uid(current_uid) {
            if current_user.primary_group_id() == group.gid()
                || group.members().contains(&current_user.name().to_owned())
            {
                style = colours.yours();
            }
        }

        TextCell::paint(style, group.name().to_owned())
    }
}

pub trait Colours {
    fn yours(&self) -> Style;
    fn not_yours(&self) -> Style;
}

#[cfg(test)]
#[allow(unused_results)]
pub mod test {
    use super::Colours;
    use fs::fields as f;
    use output::cell::TextCell;

    use ansi_term::Colour::*;
    use ansi_term::Style;
    use users::mock::MockUsers;
    use users::os::unix::GroupExt;
    use users::{Group, User};

    struct TestColours;

    impl Colours for TestColours {
        fn yours(&self) -> Style {
            Fixed(80).normal()
        }
        fn not_yours(&self) -> Style {
            Fixed(81).normal()
        }
    }

    #[test]
    fn named() {
        let mut users = MockUsers::with_current_uid(1000);
        users.add_group(Group::new(100, "folk"));

        let group = f::Group(100);
        let expected = TextCell::paint_str(Fixed(81).normal(), "folk");
        assert_eq!(expected, group.render(&TestColours, &users))
    }

    #[test]
    fn unnamed() {
        let users = MockUsers::with_current_uid(1000);

        let group = f::Group(100);
        let expected = TextCell::paint_str(Fixed(81).normal(), "100");
        assert_eq!(expected, group.render(&TestColours, &users));
    }

    #[test]
    fn primary() {
        let mut users = MockUsers::with_current_uid(2);
        users.add_user(User::new(2, "eve", 100));
        users.add_group(Group::new(100, "folk"));

        let group = f::Group(100);
        let expected = TextCell::paint_str(Fixed(80).normal(), "folk");
        assert_eq!(expected, group.render(&TestColours, &users))
    }

    #[test]
    fn secondary() {
        let mut users = MockUsers::with_current_uid(2);
        users.add_user(User::new(2, "eve", 666));

        let test_group = Group::new(100, "folk").add_member("eve");
        users.add_group(test_group);

        let group = f::Group(100);
        let expected = TextCell::paint_str(Fixed(80).normal(), "folk");
        assert_eq!(expected, group.render(&TestColours, &users))
    }

    #[test]
    fn overflow() {
        let group = f::Group(2_147_483_648);
        let expected = TextCell::paint_str(Fixed(81).normal(), "2147483648");
        assert_eq!(
            expected,
            group.render(&TestColours, &MockUsers::with_current_uid(0))
        );
    }
}
