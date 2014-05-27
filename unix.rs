use std::str::raw::from_c_str;
use std::ptr::read;

mod c {
    #![allow(non_camel_case_types)]
    extern crate libc;
    use self::libc::{
        c_char,
        c_int,
        uid_t,
        time_t
    };

    pub struct c_passwd {
        pub pw_name:    *c_char,  // login name
        pub pw_passwd:  *c_char,
        pub pw_uid:     c_int,    // user ID
        pub pw_gid:     c_int,    // group ID
        pub pw_change:  time_t,
        pub pw_class:   *c_char,
        pub pw_gecos:   *c_char,  // full name
        pub pw_dir:     *c_char,  // login dir
        pub pw_shell:   *c_char,  // login shell
        pub pw_expire:  time_t    // password expiry time
    }

    pub struct c_group {
        pub gr_name: *c_char      // group name
    }

    extern {
        pub fn getpwuid(uid: c_int) -> *c_passwd;
        pub fn getgrgid(gid: uid_t) -> *c_group;
        pub fn getuid() -> libc::c_int;
    }
}

pub fn get_user_name(uid: i32) -> Option<String> {
    let pw = unsafe { c::getpwuid(uid) };
    if pw.is_not_null() {
        return unsafe { Some(from_c_str(read(pw).pw_name)) };
    }
    else {
        return None;
    }
}

pub fn get_group_name(gid: u32) -> Option<String> {
    let gr = unsafe { c::getgrgid(gid) };
    if gr.is_not_null() {
        return unsafe { Some(from_c_str(read(gr).gr_name)) };
    }
    else {
        return None;
    }
}

pub fn get_current_user_id() -> u64 {
    unsafe { c::getuid() as u64 }
}
