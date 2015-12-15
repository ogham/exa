//! System calls for getting the terminal size.
//!
//! Getting the terminal size is performed using an ioctl command that takes
//! the file handle to the terminal -- which in this case, is stdout -- and
//! populates a structure containing the values.
//!
//! The size is needed when the user wants the output formatted into columns:
//! the default grid view, or the hybrid grid-details view.

use std::mem::zeroed;
use libc::{c_int, c_ushort, c_ulong, STDOUT_FILENO};


/// The number of rows and columns of a terminal.
struct Winsize {
    ws_row: c_ushort,
    ws_col: c_ushort,
}

// Unfortunately the actual command is not standardised...

#[cfg(any(target_os = "linux", target_os = "android"))]
static TIOCGWINSZ: c_ulong = 0x5413;

#[cfg(any(target_os = "macos",
          target_os = "ios",
          target_os = "bitrig",
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "netbsd",
          target_os = "openbsd"))]
static TIOCGWINSZ: c_ulong = 0x40087468;

extern {
    pub fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
}

/// Runs the ioctl command. Returns (0, 0) if output is not to a terminal, or
/// there is an error. (0, 0) is an invalid size to have anyway, which is why
/// it can be used as a nil value.
unsafe fn get_dimensions() -> Winsize {
    let mut window: Winsize = zeroed();
    let result = ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut window);

    if result == -1 {
        zeroed()
    }
    else {
        window
    }
}

/// Query the current processes's output, returning its width and height as a
/// number of characters. Returns `None` if the output isn't to a terminal.
pub fn dimensions() -> Option<(usize, usize)> {
    let w = unsafe { get_dimensions() };

    if w.ws_col == 0 || w.ws_row == 0 {
        None
    }
    else {
        Some((w.ws_col as usize, w.ws_row as usize))
    }
}
