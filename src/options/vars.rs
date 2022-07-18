use std::ffi::OsString;


// General variables

/// Environment variable used to colour files, both by their filesystem type
/// (symlink, socket, directory) and their file name or extension (image,
/// video, archive);
pub static LS_COLORS: &str = "LS_COLORS";

/// Environment variable used to override the width of the terminal, in
/// characters.
pub static COLUMNS: &str = "COLUMNS";

/// Environment variable used to datetime format.
pub static TIME_STYLE: &str = "TIME_STYLE";

/// Environment variable used to disable colors.
/// See: <https://no-color.org/>
pub static NO_COLOR: &str = "NO_COLOR";

// exa-specific variables

/// Environment variable used to colour exa’s interface when colours are
/// enabled. This includes all the colours that `LS_COLORS` would recognise,
/// overriding them if necessary. It can also contain exa-specific codes.
pub static EXA_COLORS: &str = "EXA_COLORS";

/// Environment variable used to switch on strict argument checking, such as
/// complaining if an argument was specified twice, or if two conflict.
/// This is meant to be so you don’t accidentally introduce the wrong
/// behaviour in a script, rather than for general command-line use.
/// Any non-empty value will turn strict mode on.
pub static EXA_STRICT: &str = "EXA_STRICT";

/// Environment variable used to make exa print out debugging information as
/// it runs. Any non-empty value will turn debug mode on.
pub static EXA_DEBUG: &str = "EXA_DEBUG";

/// Environment variable used to limit the grid-details view
/// (`--grid --long`) so it’s only activated if there’s at least the given
/// number of rows of output.
pub static EXA_GRID_ROWS: &str = "EXA_GRID_ROWS";

/// Environment variable used to specify how many spaces to print between an
/// icon and its file name. Different terminals display icons differently,
/// with 1 space bringing them too close together or 2 spaces putting them too
/// far apart, so this may be necessary depending on how they are shown.
pub static EXA_ICON_SPACING: &str = "EXA_ICON_SPACING";


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
