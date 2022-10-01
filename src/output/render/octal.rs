use ansi_term::Style;

use crate::fs::fields as f;
use crate::output::cell::TextCell;


impl f::OctalPermissions {
    fn bits_to_octal(r: bool, w: bool, x: bool) -> u8 {
        u8::from(r) * 4 + u8::from(w) * 2 + u8::from(x)
    }

    pub fn render(&self, style: Style) -> TextCell {
        let perm = &self.permissions;
        let octal_sticky = Self::bits_to_octal(perm.setuid, perm.setgid, perm.sticky);
        let octal_owner  = Self::bits_to_octal(perm.user_read, perm.user_write, perm.user_execute);
        let octal_group  = Self::bits_to_octal(perm.group_read, perm.group_write, perm.group_execute);
        let octal_other  = Self::bits_to_octal(perm.other_read, perm.other_write, perm.other_execute);

        TextCell::paint(style, format!("{}{}{}{}", octal_sticky, octal_owner, octal_group, octal_other))
    }
}


#[cfg(test)]
pub mod test {
    use crate::output::cell::TextCell;
    use crate::fs::fields as f;

    use ansi_term::Colour::*;


    #[test]
    fn normal_folder() {
        let bits = f::Permissions {
            user_read:  true, user_write:  true,  user_execute:  true, setuid: false,
            group_read: true, group_write: false, group_execute: true, setgid: false,
            other_read: true, other_write: false, other_execute: true, sticky: false,
        };

        let octal = f::OctalPermissions{ permissions: bits };

        let expected = TextCell::paint_str(Purple.bold(), "0755");
        assert_eq!(expected, octal.render(Purple.bold()));
    }

    #[test]
    fn normal_file() {
        let bits = f::Permissions {
            user_read:  true, user_write:  true,  user_execute:  false, setuid: false,
            group_read: true, group_write: false, group_execute: false, setgid: false,
            other_read: true, other_write: false, other_execute: false, sticky: false,
        };

        let octal = f::OctalPermissions{ permissions: bits };

        let expected = TextCell::paint_str(Purple.bold(), "0644");
        assert_eq!(expected, octal.render(Purple.bold()));
    }

    #[test]
    fn secret_file() {
        let bits = f::Permissions {
            user_read:  true,  user_write:  true,  user_execute:  false, setuid: false,
            group_read: false, group_write: false, group_execute: false, setgid: false,
            other_read: false, other_write: false, other_execute: false, sticky: false,
        };

        let octal = f::OctalPermissions{ permissions: bits };

        let expected = TextCell::paint_str(Purple.bold(), "0600");
        assert_eq!(expected, octal.render(Purple.bold()));
    }

    #[test]
    fn sticky1() {
        let bits = f::Permissions {
            user_read:  true, user_write:  true,  user_execute:  true, setuid: true,
            group_read: true, group_write: true,  group_execute: true, setgid: false,
            other_read: true, other_write: true,  other_execute: true, sticky: false,
        };

        let octal = f::OctalPermissions{ permissions: bits };

        let expected = TextCell::paint_str(Purple.bold(), "4777");
        assert_eq!(expected, octal.render(Purple.bold()));

    }

    #[test]
    fn sticky2() {
        let bits = f::Permissions {
            user_read:  true, user_write:  true,  user_execute:  true, setuid: false,
            group_read: true, group_write: true,  group_execute: true, setgid: true,
            other_read: true, other_write: true,  other_execute: true, sticky: false,
        };

        let octal = f::OctalPermissions{ permissions: bits };

        let expected = TextCell::paint_str(Purple.bold(), "2777");
        assert_eq!(expected, octal.render(Purple.bold()));
    }

    #[test]
    fn sticky3() {
        let bits = f::Permissions {
            user_read:  true, user_write:  true,  user_execute:  true, setuid: false,
            group_read: true, group_write: true,  group_execute: true, setgid: false,
            other_read: true, other_write: true,  other_execute: true, sticky: true,
        };

        let octal = f::OctalPermissions{ permissions: bits };

        let expected = TextCell::paint_str(Purple.bold(), "1777");
        assert_eq!(expected, octal.render(Purple.bold()));
    }
}
