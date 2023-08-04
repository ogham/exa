use ansi_term::Style;
use phf;

use crate::fs::File;

#[non_exhaustive]
struct Icons;

impl Icons {
    const AUDIO: char          = '\u{f001}';  // 
    const IMAGE: char          = '\u{f1c5}';  // 
    const VIDEO: char          = '\u{f03d}';  // 
    const COMPRESSED: char     = '\u{f410}';  // 
    const SHELL: char          = '\u{f489}';  // 
    const GIT: char            = '\u{f1d3}';  // 
    const MARKDOWN: char       = '\u{f48a}';  // 
    const DISK_IMAGE: char     = '\u{e271}';  // 
    const CONFIG: char         = '\u{e615}';  // 
    const JSON: char           = '\u{e60b}';  // 
    const C_LANG: char         = '\u{e61e}';  // 
    const CPP_LANG: char       = '\u{e61d}';  // 
    const CSHARP_LANG: char    = '\u{f031b}'; // 󰌛
    const FSHARP_LANG: char    = '\u{e7a7}';  // 
    const GO_LANG: char        = '\u{e626}';  // 
    const JAVA_LANG: char      = '\u{e256}';  // 
    const PERL_LANG: char      = '\u{e769}';  // 
    const PYTHON_LANG: char    = '\u{e606}';  // 
    const R_LANG: char         = '\u{f25d}';  // 
    const RUBY_LANG: char      = '\u{e21e}';  // 
    const RUBYRAILS_LANG: char = '\u{e73b}';  // 
    const RUST_LANG: char      = '\u{e7a8}';  // 
    const TEX_LANG: char       = '\u{f034}';  // 
    const APPLE: char          = '\u{f179}';  // 
    const WINDOWS: char        = '\u{e70f}';  // 
    const HTML5: char          = '\u{f13b}';  // 
    const FONT: char           = '\u{f031}';  // 
    const DATABASE: char       = '\u{f1c0}';  // 
    const DOCUMENT: char       = '\u{f1c2}';  // 
    const SHEET: char          = '\u{f1c3}';  // 
    const SLIDE: char          = '\u{f1c4}';  // 
}

// See build.rs for FILENAME_ICONS and EXTENSION_ICONS
include!(concat!(env!("OUT_DIR"), "/icon_maps.rs"));

/// Converts the style used to paint a file name into the style that should be
/// used to paint an icon.
///
/// - The background colour should be preferred to the foreground colour, as
///   if one is set, it’s the more “obvious” colour choice.
/// - If neither is set, just use the default style.
/// - Attributes such as bold or underline should not be used to paint the
///   icon, as they can make it look weird.
pub fn iconify_style(style: Style) -> Style {
    style.background.or(style.foreground)
         .map(Style::from)
         .unwrap_or_default()
}

pub fn icon_for_file(file: &File<'_>) -> char {
    if let Some(icon) = FILENAME_ICONS.get(file.name.as_str()) {
        *icon
    } else if file.points_to_directory() {
        '\u{f115}' // 
    } else if let Some(ext) = file.ext.as_ref() {
        match EXTENSION_ICONS.get(ext.as_str()) {
            Some(icon) => *icon,
            None              => '\u{f15b}' // 
        }
    } else {
        '\u{f016}' // 
    }
}
