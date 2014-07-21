use std::str::raw::from_c_str;
use std::ptr::read;
use std::collections::hashmap::HashMap;

mod c {
    #![allow(non_camel_case_types)]
    extern crate libc;
    pub use self::libc::{
        c_char,
        c_int,
        uid_t,
        gid_t,
        time_t
    };

    pub struct c_passwd {
        pub pw_name:    *const c_char,  // login name
        pub pw_passwd:  *const c_char,
        pub pw_uid:     c_int,    // user ID
        pub pw_gid:     c_int,    // group ID
        pub pw_change:  time_t,
        pub pw_class:   *const c_char,
        pub pw_gecos:   *const c_char,  // full name
        pub pw_dir:     *const c_char,  // login dir
        pub pw_shell:   *const c_char,  // login shell
        pub pw_expire:  time_t    // password expiry time
    }

    pub struct c_group {
        pub gr_name:   *const c_char,   // group name
        pub gr_passwd: *const c_char,   // password
        pub gr_gid:    gid_t,     // group id
        pub gr_mem:    *const *const c_char,  // names of users in the group
    }

    extern {
        pub fn getpwuid(uid: c_int) -> *const c_passwd;
        pub fn getgrgid(gid: uid_t) -> *const c_group;
        pub fn getuid() -> libc::c_int;
    }
}
pub struct Unix {
    user_names:    HashMap<u32, Option<String>>,  // mapping of user IDs to user names
    group_names:   HashMap<u32, Option<String>>,  // mapping of groups IDs to group names
    groups:        HashMap<u32, bool>,            // mapping of group IDs to whether the current user is a member
    pub uid:       u32,                           // current user's ID
    pub username:  String,                        // current user's name
}

impl Unix {
    pub fn empty_cache() -> Unix {
        let uid = unsafe { c::getuid() };
        let infoptr = unsafe { c::getpwuid(uid as i32) };
        let info = unsafe { infoptr.to_option().unwrap() };  // the user has to have a name

        let username = unsafe { from_c_str(info.pw_name) };

        let mut user_names = HashMap::new();
        user_names.insert(uid as u32, Some(username.clone()));

        // Unix groups work like this: every group has a list of
        // users, referred to by their names. But, every user also has
        // a primary group, which isn't in this list. So handle this
        // case immediately after we look up the user's details.
        let mut groups = HashMap::new();
        groups.insert(info.pw_gid as u32, true);

        Unix {
            user_names:  user_names,
            group_names: HashMap::new(),
            uid:         uid as u32,
            username:    username,
            groups:      groups,
        }
    }

    pub fn get_user_name(&self, uid: u32) -> Option<String> {
        self.user_names.get(&uid).clone()
    }

    pub fn get_group_name(&self, gid: u32) -> Option<String> {
        self.group_names.get(&gid).clone()
    }

    pub fn is_group_member(&self, gid: u32) -> bool {
        *self.groups.get(&gid)
    }

    pub fn load_user(&mut self, uid: u32) {
        let pw = unsafe { c::getpwuid(uid as i32) };
        if pw.is_not_null() {
            let username = unsafe { Some(from_c_str(read(pw).pw_name)) };
            self.user_names.insert(uid, username);
        }
        else {
            self.user_names.insert(uid, None);
        }
    }

    fn group_membership(group: *const *const i8, uname: &String) -> bool {
        let mut i = 0;

        // The list of members is a pointer to a pointer of
        // characters, terminated by a null pointer. So the first call
        // to `to_option` will always succeed, as that memory is
        // guaranteed to be there (unless we go past the end of RAM).
        // The second call will return None if it's a null pointer.

        loop {
            match unsafe { group.offset(i).to_option().unwrap().to_option() } {
                Some(username) => {
                    if unsafe { from_c_str(username) } == *uname {
                        return true;
                    }
                }
                None => {
                    return false;
                }
            }
            i += 1;
        }
    }

    pub fn load_group(&mut self, gid: u32) {
        match unsafe { c::getgrgid(gid).to_option() } {
            None => {
                self.group_names.find_or_insert(gid, None);
                self.groups.find_or_insert(gid, false);
            },
            Some(r) => {
                let group_name = unsafe { Some(from_c_str(r.gr_name)) };
                self.groups.find_or_insert(gid, Unix::group_membership(r.gr_mem, &self.username));
                self.group_names.find_or_insert(gid, group_name);
            }
        }
        
    }
}

