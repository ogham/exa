mod c {
    use std::mem::zeroed;
    use libc::{c_int, c_ushort, c_ulong, STDOUT_FILENO};

    // Getting the terminal size is done using an ioctl command that
    // takes the file handle to the terminal (which in our case is
    // stdout), and populates a structure with the values.

    pub struct Winsize {
        pub ws_row: c_ushort,
        pub ws_col: c_ushort,
    }

    // Unfortunately the actual command is not standardised...

    #[cfg(any(target_os = "linux", target_os = "android"))]
    static TIOCGWINSZ: c_ulong = 0x5413;

    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "dragonfly"))]
    static TIOCGWINSZ: c_ulong = 0x40087468;

    extern {
        pub fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
    }

    pub unsafe fn dimensions() -> Winsize {
        let mut window: Winsize = zeroed();
        ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut window);
        window
    }
}

/// Query the current processes's output, returning its width and height as a
/// number of characters. Returns None if the output isn't to a terminal.
pub fn dimensions() -> Option<(usize, usize)> {
    let w = unsafe { c::dimensions() };

    if w.ws_col == 0 || w.ws_row == 0 {
        None
    }
    else {
        Some((w.ws_col as usize, w.ws_row as usize))
    }
}
