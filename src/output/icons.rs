use ansi_term::Style;
use phf;

use crate::fs::File;

#[non_exhaustive]
struct Icons;

impl Icons {
    const AUDIO: char           = '\u{f075a}'; // 󰝚
    const CALENDAR: char        = '\u{eab0}';  // 
    const COMPRESSED: char      = '\u{f410}';  // 
    const CONFIG: char          = '\u{e615}';  // 
    const CONFIG_FOLDER: char   = '\u{e5fc}';  // 
    const DATABASE: char        = '\u{f1c0}';  // 
    const DISK_IMAGE: char      = '\u{e271}';  // 
    const DOCUMENT: char        = '\u{f1c2}';  // 
    const EMACS: char           = '\u{e632}';  // 
    const FONT: char            = '\u{f031}';  // 
    const GIT: char             = '\u{f1d3}';  // 
    const GRADLE: char          = '\u{e660}';  // 
    const GRUNT: char           = '\u{e611}';  // 
    const GULP: char            = '\u{e610}';  // 
    const HTML5: char           = '\u{f13b}';  // 
    const IMAGE: char           = '\u{f1c5}';  // 
    const JSON: char            = '\u{e60b}';  // 
    const LANG_C: char          = '\u{e61e}';  // 
    const LANG_CPP: char        = '\u{e61d}';  // 
    const LANG_CSHARP: char     = '\u{f031b}'; // 󰌛
    const LANG_FSHARP: char     = '\u{e7a7}';  // 
    const LANG_GO: char         = '\u{e627}';  // 
    const LANG_JAVA: char       = '\u{e256}';  // 
    const LANG_JAVASCRIPT: char = '\u{e74e}';  // 
    const LANG_OCAML: char      = '\u{e67a}';  // 
    const LANG_PERL: char       = '\u{e769}';  // 
    const LANG_PHP: char        = '\u{f031f}'; // 󰌟
    const LANG_PYTHON: char     = '\u{e606}';  // 
    const LANG_R: char          = '\u{f25d}';  // 
    const LANG_RUBY: char       = '\u{e21e}';  // 
    const LANG_RUBYRAILS: char  = '\u{e73b}';  // 
    const LANG_RUST: char       = '\u{e7a8}';  // 
    const LANG_TEX: char        = '\u{e69b}';  // 
    const LANG_TYPESCRIPT: char = '\u{e628}';  // 
    const LOCK: char            = '\u{f023}';  // 
    const LICENSE: char         = '\u{e60a}';  // 
    const MAKE: char            = '\u{e673}';  // 
    const MARKDOWN: char        = '\u{f48a}';  // 
    const NPM: char             = '\u{e71e}';  // 
    const NPM_FOLDER: char      = '\u{e5fa}';  // 
    const OS_APPLE: char        = '\u{f179}';  // 
    const OS_LINUX: char        = '\u{f17c}';  // 
    const OS_WINDOWS: char      = '\u{f17a}';  // 
    const OS_WINDOWS_CMD: char  = '\u{ebc4}';  // 
    const PLAYLIST: char        = '\u{f0cb9}'; // 󰲹
    const PRIVATE_KEY: char     = '\u{f0306}'; // 󰌆
    const PUBLIC_KEY: char      = '\u{f0dd6}'; // 󰷖
    const SHEET: char           = '\u{f1c3}';  // 
    const SHELL: char           = '\u{f489}';  // 
    const SLIDE: char           = '\u{f1c4}';  // 
    const TERRAFORM: char       = '\u{f1062}'; // 󱁢
    const VIDEO: char           = '\u{f008}';  // 
    const VIM: char             = '\u{e62b}';  // 
}

// See build.rs for FILENAME_ICONS, EXTENSION_ICONS, and DIRECTORY_ICONS
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
    if file.points_to_directory() {
        *DIRECTORY_ICONS.get(file.name.as_str()).unwrap_or(&'\u{e5ff}') // 
    } else if let Some(icon) = FILENAME_ICONS.get(file.name.as_str()) {
        *icon
    } else if let Some(ext) = file.ext.as_ref() {
        *EXTENSION_ICONS.get(ext.as_str()).unwrap_or(&'\u{f15b}') // 
    } else {
        '\u{f016}' // 
    }
}
