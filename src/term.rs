mod c {
    #![allow(non_camel_case_types)]
    extern crate libc;
    pub use self::libc::{
        c_int,
        c_ushort,
        c_ulong,
        STDOUT_FILENO,
    };
    use std::mem::zeroed;

    // Getting the terminal size is done using an ioctl command that
    // takes the file handle to the terminal (which in our case is
    // stdout), and populates a structure with the values.

    pub struct winsize {
        pub ws_row: c_ushort,
        pub ws_col: c_ushort,
    }

    // Unfortunately the actual command is not standardised...

    #[cfg(any(target_os = "linux", target_os = "android"))]
    static TIOCGWINSZ: c_ulong = 0x5413;

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    static TIOCGWINSZ: c_ulong = 0x40087468;

    extern {
        pub fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
    }

    pub fn dimensions() -> winsize {
        unsafe {
            let mut window: winsize = zeroed();
            ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut window as *mut winsize);
            window
        }
    }
}

pub fn dimensions() -> Option<(uint, uint)> {
    let w = c::dimensions();

    // If either of the dimensions is 0 then the command failed,
    // usually because output isn't to a terminal (instead to a file
    // or pipe or something)
    if w.ws_col == 0 || w.ws_row == 0 {
        None
    }
    else {
        Some((w.ws_col as uint, w.ws_row as uint))
    }
}
