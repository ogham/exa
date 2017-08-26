use ansi_term::Style;
use users::Users;

use fs::fields as f;
use output::cell::TextCell;



impl f::User {
    pub fn render<C: Colours, U: Users>(&self, colours: &C, users: &U) -> TextCell {
        let user_name = match users.get_user_by_uid(self.0) {
            Some(user)  => user.name().to_owned(),
            None        => self.0.to_string(),
        };

        let style =  if users.get_current_uid() == self.0 { colours.you() }
                                                     else { colours.someone_else() };
        TextCell::paint(style, user_name)
    }
}


pub trait Colours {
    fn you(&self) -> Style;
    fn someone_else(&self) -> Style;
}


#[cfg(test)]
#[allow(unused_results)]
pub mod test {
    use super::Colours;
    use fs::fields as f;
    use output::cell::TextCell;

    use users::User;
    use users::mock::MockUsers;
    use ansi_term::Colour::*;
    use ansi_term::Style;


    struct TestColours;

    impl Colours for TestColours {
        fn you(&self)          -> Style { Red.bold() }
        fn someone_else(&self) -> Style { Blue.underline() }
    }


    #[test]
    fn named() {
        let mut users = MockUsers::with_current_uid(1000);
        users.add_user(User::new(1000, "enoch", 100));

        let user = f::User(1000);
        let expected = TextCell::paint_str(Red.bold(), "enoch");
        assert_eq!(expected, user.render(&TestColours, &users))
    }

    #[test]
    fn unnamed() {
        let users = MockUsers::with_current_uid(1000);

        let user = f::User(1000);
        let expected = TextCell::paint_str(Red.bold(), "1000");
        assert_eq!(expected, user.render(&TestColours, &users));
    }

    #[test]
    fn different_named() {
        let mut users = MockUsers::with_current_uid(0);
        users.add_user(User::new(1000, "enoch", 100));

        let user = f::User(1000);
        let expected = TextCell::paint_str(Blue.underline(), "enoch");
        assert_eq!(expected, user.render(&TestColours, &users));
    }

    #[test]
    fn different_unnamed() {
        let user = f::User(1000);
        let expected = TextCell::paint_str(Blue.underline(), "1000");
        assert_eq!(expected, user.render(&TestColours, &MockUsers::with_current_uid(0)));
    }

    #[test]
    fn overflow() {
        let user = f::User(2_147_483_648);
        let expected = TextCell::paint_str(Blue.underline(), "2147483648");
        assert_eq!(expected, user.render(&TestColours, &MockUsers::with_current_uid(0)));
    }
}
