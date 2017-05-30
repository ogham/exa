use users::Users;

use fs::fields as f;
use output::colours::Colours;
use output::cell::TextCell;


impl f::User {
    pub fn render(&self, colours: &Colours, users: &Users) -> TextCell {
        let user_name = match users.get_user_by_uid(self.0) {
            Some(user)  => user.name().to_owned(),
            None        => self.0.to_string(),
        };

        let style = if users.get_current_uid() == self.0 { colours.users.user_you }
                                                    else { colours.users.user_someone_else };
        TextCell::paint(style, user_name)
    }
}

#[cfg(test)]
#[allow(unused_results)]
pub mod test {
    use output::details::Details;

    use fs::fields as f;
    use output::cell::TextCell;

    use users::User;
    use users::mock::MockUsers;
    use ansi_term::Colour::*;

    #[test]
    fn named() {
        let mut details = Details::default();
        details.colours.users.user_you = Red.bold();

        let mut users = MockUsers::with_current_uid(1000);
        users.add_user(User::new(1000, "enoch", 100));

        let user = f::User(1000);
        let expected = TextCell::paint_str(Red.bold(), "enoch");
        assert_eq!(expected, user.render(&details.colours, &users))
    }

    #[test]
    fn unnamed() {
        let mut details = Details::default();
        details.colours.users.user_you = Cyan.bold();

        let users = MockUsers::with_current_uid(1000);

        let user = f::User(1000);
        let expected = TextCell::paint_str(Cyan.bold(), "1000");
        assert_eq!(expected, user.render(&details.colours, &users));
    }

    #[test]
    fn different_named() {
        let mut details = Details::default();
        details.colours.users.user_someone_else = Green.bold();

        let mut users = MockUsers::with_current_uid(0);
        users.add_user(User::new(1000, "enoch", 100));

        let user = f::User(1000);
        let expected = TextCell::paint_str(Green.bold(), "enoch");
        assert_eq!(expected, user.render(&details.colours, &users));
    }

    #[test]
    fn different_unnamed() {
        let mut details = Details::default();
        details.colours.users.user_someone_else = Red.normal();

        let user = f::User(1000);
        let expected = TextCell::paint_str(Red.normal(), "1000");
        assert_eq!(expected, user.render(&details.colours, &MockUsers::with_current_uid(0)));
    }

    #[test]
    fn overflow() {
        let mut details = Details::default();
        details.colours.users.user_someone_else = Blue.underline();

        let user = f::User(2_147_483_648);
        let expected = TextCell::paint_str(Blue.underline(), "2147483648");
        assert_eq!(expected, user.render(&details.colours, &MockUsers::with_current_uid(0)));
    }
}
