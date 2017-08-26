use std::ffi::OsString;


// General variables

/// Environment variable used to colour files, both by their filesystem type
/// (symlink, socket, directory) and their file name or extension (image,
/// video, archive);
pub static LS_COLORS: &str = "LS_COLORS";

/// Environment variable used to override the width of the terminal, in
/// characters.
pub static COLUMNS: &str = "COLUMNS";


// exa-specific variables

/// Environment variable used to colour exa’s interface when colours are
/// enabled. This includes all the colours that LS_COLORS would recognise,
/// overriding them if necessary. It can also contain exa-specific codes.
pub static EXA_COLORS: &str = "EXA_COLORS";

/// Environment variable used to switch on strict argument checking, such as
/// complaining if an argument was specified twice, or if two conflict.
/// This is meant to be so you don’t accidentally introduce the wrong
/// behaviour in a script, rather than for general command-line use.
pub static EXA_STRICT: &str = "EXA_STRICT";

/// Environment variable used to limit the grid-details view
/// (`--grid --long`) so it’s only activated if there’s at least the given
/// number of rows of output.
pub static EXA_GRID_ROWS: &str = "EXA_GRID_ROWS";



/// Mockable wrapper for `std::env::var_os`.
pub trait Vars {
    fn get(&self, name: &'static str) -> Option<OsString>;
}


// Test impl that just returns the value it has.
#[cfg(test)]
impl Vars for Option<OsString> {
    fn get(&self, _name: &'static str) -> Option<OsString> {
        self.clone()
    }
}
