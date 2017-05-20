use users::{Users, Groups};

use fs::fields as f;
use output::cell::TextCell;
use output::details::Table;


impl<'a, U: Users+Groups+'a> Table<'a, U> {

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

    pub fn render_group(&self, group: f::Group) -> TextCell {
        use users::os::unix::GroupExt;

        let mut style = self.opts.colours.users.group_not_yours;

        let users = self.env.users();
        let group = match users.get_group_by_gid(group.0) {
            Some(g) => (*g).clone(),
            None    => return TextCell::paint(style, group.0.to_string()),
        };

        let current_uid = users.get_current_uid();
        if let Some(current_user) = users.get_user_by_uid(current_uid) {
            if current_user.primary_group_id() == group.gid()
            || group.members().contains(&current_user.name().to_owned()) {
                style = self.opts.colours.users.group_yours;
            }
        }

        TextCell::paint(style, group.name().to_owned())
    }
}


#[cfg(test)]
pub mod test {
    pub use output::details::{Table, Environment, Details};
    pub use output::details::test::new_table;
    pub use std::sync::Mutex;

    pub use fs::{File, fields as f};
    pub use output::column::{Column, Columns};
    pub use output::cell::TextCell;

    pub use users::{User, Group, uid_t, gid_t};
    pub use users::mock::MockUsers;
    pub use users::os::unix::{UserExt, GroupExt};
    pub use datetime::fmt::DateFormat;
    pub use ansi_term::Style;
    pub use ansi_term::Colour::*;

    mod users {
        #![allow(unused_results)]
        use super::*;

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

    mod groups {
        #![allow(unused_results)]
        use super::*;

        #[test]
        fn named() {
            let columns = Columns::default().for_dir(None);
            let mut details = Details::default();
            details.colours.users.group_not_yours = Fixed(101).normal();

            let mut users = MockUsers::with_current_uid(1000);
            users.add_group(Group::new(100, "folk"));
            let table = new_table(&columns, &details, users);

            let group = f::Group(100);
            let expected = TextCell::paint_str(Fixed(101).normal(), "folk");
            assert_eq!(expected, table.render_group(group))
        }

        #[test]
        fn unnamed() {
            let columns = Columns::default().for_dir(None);
            let mut details = Details::default();
            details.colours.users.group_not_yours = Fixed(87).normal();

            let users = MockUsers::with_current_uid(1000);
            let table = new_table(&columns, &details, users);

            let group = f::Group(100);
            let expected = TextCell::paint_str(Fixed(87).normal(), "100");
            assert_eq!(expected, table.render_group(group));
        }

        #[test]
        fn primary() {
            let columns = Columns::default().for_dir(None);
            let mut details = Details::default();
            details.colours.users.group_yours = Fixed(64).normal();

            let mut users = MockUsers::with_current_uid(2);
            users.add_user(User::new(2, "eve", 100));
            users.add_group(Group::new(100, "folk"));

            let table = new_table(&columns, &details, users);

            let group = f::Group(100);
            let expected = TextCell::paint_str(Fixed(64).normal(), "folk");
            assert_eq!(expected, table.render_group(group))
        }

        #[test]
        fn secondary() {
            let columns = Columns::default().for_dir(None);
            let mut details = Details::default();
            details.colours.users.group_yours = Fixed(31).normal();

            let mut users = MockUsers::with_current_uid(2);
            users.add_user(User::new(2, "eve", 666));

            let test_group = Group::new(100, "folk").add_member("eve");
            users.add_group(test_group);

            let table = new_table(&columns, &details, users);

            let group = f::Group(100);
            let expected = TextCell::paint_str(Fixed(31).normal(), "folk");
            assert_eq!(expected, table.render_group(group))
        }

        #[test]
        fn overflow() {
            let columns = Columns::default().for_dir(None);
            let mut details = Details::default();
            details.colours.users.group_not_yours = Blue.underline();

            let table = new_table(&columns, &details, MockUsers::with_current_uid(0));

            let group = f::Group(2_147_483_648);
            let expected = TextCell::paint_str(Blue.underline(), "2147483648");
            assert_eq!(expected, table.render_group(group));
        }
    }
}
