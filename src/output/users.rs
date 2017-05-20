use users::Users;

use fs::fields as f;
use output::cell::TextCell;
use output::details::Table;


impl<'a, U: Users+'a> Table<'a, U> {
    pub fn render_user(&self, user: f::User) -> TextCell {
        let users = self.env.users();

        let user_name = match users.get_user_by_uid(user.0) {
            Some(user)  => user.name().to_owned(),
            None        => user.0.to_string(),
        };

        let style = if users.get_current_uid() == user.0 { self.opts.colours.users.user_you }
                                                    else { self.opts.colours.users.user_someone_else };
        TextCell::paint(style, user_name)
    }
}

#[cfg(test)]
#[allow(unused_results)]
pub mod test {
    use output::details::Details;
    use output::details::test::new_table;

    use fs::fields as f;
    use output::column::Columns;
    use output::cell::TextCell;

    use users::User;
    use users::mock::MockUsers;
    use ansi_term::Colour::*;

    #[test]
    fn named() {
        let columns = Columns::default().for_dir(None);
        let mut details = Details::default();
        details.colours.users.user_you = Red.bold();

        let mut users = MockUsers::with_current_uid(1000);
        users.add_user(User::new(1000, "enoch", 100));

        let table = new_table(&columns, &details, users);

        let user = f::User(1000);
        let expected = TextCell::paint_str(Red.bold(), "enoch");
        assert_eq!(expected, table.render_user(user))
    }

    #[test]
    fn unnamed() {
        let columns = Columns::default().for_dir(None);
        let mut details = Details::default();
        details.colours.users.user_you = Cyan.bold();

        let users = MockUsers::with_current_uid(1000);

        let table = new_table(&columns, &details, users);

        let user = f::User(1000);
        let expected = TextCell::paint_str(Cyan.bold(), "1000");
        assert_eq!(expected, table.render_user(user));
    }

    #[test]
    fn different_named() {
        let columns = Columns::default().for_dir(None);
        let mut details = Details::default();
        details.colours.users.user_someone_else = Green.bold();

        let table = new_table(&columns, &details, MockUsers::with_current_uid(0));
        table.env.users().add_user(User::new(1000, "enoch", 100));

        let user = f::User(1000);
        let expected = TextCell::paint_str(Green.bold(), "enoch");
        assert_eq!(expected, table.render_user(user));
    }

    #[test]
    fn different_unnamed() {
        let columns = Columns::default().for_dir(None);
        let mut details = Details::default();
        details.colours.users.user_someone_else = Red.normal();

        let table = new_table(&columns, &details, MockUsers::with_current_uid(0));

        let user = f::User(1000);
        let expected = TextCell::paint_str(Red.normal(), "1000");
        assert_eq!(expected, table.render_user(user));
    }

    #[test]
    fn overflow() {
        let columns = Columns::default().for_dir(None);
        let mut details = Details::default();
        details.colours.users.user_someone_else = Blue.underline();

        let table = new_table(&columns, &details, MockUsers::with_current_uid(0));

        let user = f::User(2_147_483_648);
        let expected = TextCell::paint_str(Blue.underline(), "2147483648");
        assert_eq!(expected, table.render_user(user));
    }
}
