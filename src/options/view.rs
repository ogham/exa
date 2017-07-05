use std::env::var_os;

use getopts;

use output::Colours;
use output::{grid, details};
use output::table::{TimeTypes, SizeFormat, Options as TableOptions};
use output::file_name::Classify;
use options::Misfire;
use fs::feature::xattr;


/// The **view** contains all information about how to format output.
#[derive(Debug)]
pub struct View {
    pub mode: Mode,
    pub colours: Colours,
    pub classify: Classify,
}

impl View {

    /// Determine which view to use and all of that view’s arguments.
    pub fn deduce(matches: &getopts::Matches) -> Result<View, Misfire> {
        let mode     = Mode::deduce(matches)?;
        let colours  = Colours::deduce(matches)?;
        let classify = Classify::deduce(matches);
        Ok(View { mode, colours, classify })
    }
}


/// The **mode** is the “type” of output.
#[derive(Debug)]
pub enum Mode {
    Grid(grid::Options),
    Details(details::Options),
    GridDetails(grid::Options, details::Options),
    Lines,
}

impl Mode {

    /// Determine the mode from the command-line arguments.
    pub fn deduce(matches: &getopts::Matches) -> Result<Mode, Misfire> {
        use options::misfire::Misfire::*;

        let long = || {
            if matches.opt_present("across") && !matches.opt_present("grid") {
                Err(Useless("across", true, "long"))
            }
            else if matches.opt_present("oneline") {
                Err(Useless("oneline", true, "long"))
            }
            else {
                Ok(details::Options {
                    columns: Some(TableOptions::deduce(matches)?),
                    header: matches.opt_present("header"),
                    xattr: xattr::ENABLED && matches.opt_present("extended"),
                })
            }
        };

        let long_options_scan = || {
            for option in &[ "binary", "bytes", "inode", "links", "header", "blocks", "time", "group" ] {
                if matches.opt_present(option) {
                    return Err(Useless(option, false, "long"));
                }
            }

            if cfg!(feature="git") && matches.opt_present("git") {
                Err(Useless("git", false, "long"))
            }
            else if matches.opt_present("level") && !matches.opt_present("recurse") && !matches.opt_present("tree") {
                Err(Useless2("level", "recurse", "tree"))
            }
            else if xattr::ENABLED && matches.opt_present("extended") {
                Err(Useless("extended", false, "long"))
            }
            else {
                Ok(())
            }
        };

        let other_options_scan = || {
            if let Some(width) = TerminalWidth::deduce()?.width() {
                if matches.opt_present("oneline") {
                    if matches.opt_present("across") {
                        Err(Useless("across", true, "oneline"))
                    }
                    else {
                        Ok(Mode::Lines)
                    }
                }
                else if matches.opt_present("tree") {
                    let details = details::Options {
                        columns: None,
                        header: false,
                        xattr: false,
                    };

                    Ok(Mode::Details(details))
                }
                else {
                    let grid = grid::Options {
                        across: matches.opt_present("across"),
                        console_width: width,
                    };

                    Ok(Mode::Grid(grid))
                }
            }
            else {
                // If the terminal width couldn’t be matched for some reason, such
                // as the program’s stdout being connected to a file, then
                // fallback to the lines view.

                if matches.opt_present("tree") {
                    let details = details::Options {
                        columns: None,
                        header: false,
                        xattr: false,
                    };

                    Ok(Mode::Details(details))
                }
                else {
                    Ok(Mode::Lines)
                }
            }
        };

        if matches.opt_present("long") {
            let details = long()?;
            if matches.opt_present("grid") {
                match other_options_scan()? {
                    Mode::Grid(grid)  => return Ok(Mode::GridDetails(grid, details)),
                    others            => return Ok(others),
                };
            }
            else {
                return Ok(Mode::Details(details));
            }
        }

        long_options_scan()?;

        other_options_scan()
    }
}


/// The width of the terminal requested by the user.
#[derive(PartialEq, Debug)]
enum TerminalWidth {

    /// The user requested this specific number of columns.
    Set(usize),

    /// The terminal was found to have this number of columns.
    Terminal(usize),

    /// The user didn’t request any particular terminal width.
    Unset,
}

impl TerminalWidth {

    /// Determine a requested terminal width from the command-line arguments.
    ///
    /// Returns an error if a requested width doesn’t parse to an integer.
    fn deduce() -> Result<TerminalWidth, Misfire> {
        if let Some(columns) = var_os("COLUMNS").and_then(|s| s.into_string().ok()) {
            match columns.parse() {
                Ok(width)  => Ok(TerminalWidth::Set(width)),
                Err(e)     => Err(Misfire::FailedParse(e)),
            }
        }
        else if let Some(width) = *TERM_WIDTH {
            Ok(TerminalWidth::Terminal(width))
        }
        else {
            Ok(TerminalWidth::Unset)
        }
    }

    fn width(&self) -> Option<usize> {
        match *self {
            TerminalWidth::Set(width)       |
            TerminalWidth::Terminal(width)  => Some(width),
            TerminalWidth::Unset            => None,
        }
    }
}


impl TableOptions {
    fn deduce(matches: &getopts::Matches) -> Result<Self, Misfire> {
        Ok(TableOptions {
            size_format: SizeFormat::deduce(matches)?,
            time_types:  TimeTypes::deduce(matches)?,
            inode:  matches.opt_present("inode"),
            links:  matches.opt_present("links"),
            blocks: matches.opt_present("blocks"),
            group:  matches.opt_present("group"),
            git:    cfg!(feature="git") && matches.opt_present("git"),
        })
    }
}


impl SizeFormat {

    /// Determine which file size to use in the file size column based on
    /// the user’s options.
    ///
    /// The default mode is to use the decimal prefixes, as they are the
    /// most commonly-understood, and don’t involve trying to parse large
    /// strings of digits in your head. Changing the format to anything else
    /// involves the `--binary` or `--bytes` flags, and these conflict with
    /// each other.
    fn deduce(matches: &getopts::Matches) -> Result<SizeFormat, Misfire> {
        let binary = matches.opt_present("binary");
        let bytes  = matches.opt_present("bytes");

        match (binary, bytes) {
            (true,  true )  => Err(Misfire::Conflict("binary", "bytes")),
            (true,  false)  => Ok(SizeFormat::BinaryBytes),
            (false, true )  => Ok(SizeFormat::JustBytes),
            (false, false)  => Ok(SizeFormat::DecimalBytes),
        }
    }
}


impl TimeTypes {

    /// Determine which of a file’s time fields should be displayed for it
    /// based on the user’s options.
    ///
    /// There are two separate ways to pick which fields to show: with a
    /// flag (such as `--modified`) or with a parameter (such as
    /// `--time=modified`). An error is signaled if both ways are used.
    ///
    /// It’s valid to show more than one column by passing in more than one
    /// option, but passing *no* options means that the user just wants to
    /// see the default set.
    fn deduce(matches: &getopts::Matches) -> Result<TimeTypes, Misfire> {
        let possible_word = matches.opt_str("time");
        let modified = matches.opt_present("modified");
        let created  = matches.opt_present("created");
        let accessed = matches.opt_present("accessed");

        if let Some(word) = possible_word {
            if modified {
                return Err(Misfire::Useless("modified", true, "time"));
            }
            else if created {
                return Err(Misfire::Useless("created", true, "time"));
            }
            else if accessed {
                return Err(Misfire::Useless("accessed", true, "time"));
            }

            static TIMES: &[& str] = &["modified", "accessed", "created"];
            match &*word {
                "mod" | "modified"  => Ok(TimeTypes { accessed: false, modified: true,  created: false }),
                "acc" | "accessed"  => Ok(TimeTypes { accessed: true,  modified: false, created: false }),
                "cr"  | "created"   => Ok(TimeTypes { accessed: false, modified: false, created: true  }),
                otherwise           => Err(Misfire::bad_argument("time", otherwise, TIMES))
            }
        }
        else if modified || created || accessed {
            Ok(TimeTypes { accessed, modified, created })
        }
        else {
            Ok(TimeTypes::default())
        }
    }
}


/// Under what circumstances we should display coloured, rather than plain,
/// output to the terminal.
///
/// By default, we want to display the colours when stdout can display them.
/// Turning them on when output is going to, say, a pipe, would make programs
/// such as `grep` or `more` not work properly. So the `Automatic` mode does
/// this check and only displays colours when they can be truly appreciated.
#[derive(PartialEq, Debug)]
enum TerminalColours {

    /// Display them even when output isn’t going to a terminal.
    Always,

    /// Display them when output is going to a terminal, but not otherwise.
    Automatic,

    /// Never display them, even when output is going to a terminal.
    Never,
}

impl Default for TerminalColours {
    fn default() -> TerminalColours {
        TerminalColours::Automatic
    }
}

impl TerminalColours {

    /// Determine which terminal colour conditions to use.
    fn deduce(matches: &getopts::Matches) -> Result<TerminalColours, Misfire> {
        const COLOURS: &[&str] = &["always", "auto", "never"];

        if let Some(word) = matches.opt_str("color").or_else(|| matches.opt_str("colour")) {
            match &*word {
                "always"              => Ok(TerminalColours::Always),
                "auto" | "automatic"  => Ok(TerminalColours::Automatic),
                "never"               => Ok(TerminalColours::Never),
                otherwise             => Err(Misfire::bad_argument("color", otherwise, COLOURS))
            }
        }
        else {
            Ok(TerminalColours::default())
        }
    }
}


impl Colours {
    fn deduce(matches: &getopts::Matches) -> Result<Colours, Misfire> {
        use self::TerminalColours::*;

        let tc = TerminalColours::deduce(matches)?;
        if tc == Always || (tc == Automatic && TERM_WIDTH.is_some()) {
            let scale = matches.opt_present("color-scale") || matches.opt_present("colour-scale");
            Ok(Colours::colourful(scale))
        }
        else {
            Ok(Colours::plain())
        }
    }
}



impl Classify {
    fn deduce(matches: &getopts::Matches) -> Classify {
        if matches.opt_present("classify") { Classify::AddFileIndicators }
                                      else { Classify::JustFilenames }
    }
}


// Gets, then caches, the width of the terminal that exa is running in.
// This gets used multiple times above, with no real guarantee of order,
// so it’s easier to just cache it the first time it runs.
lazy_static! {
    static ref TERM_WIDTH: Option<usize> = {
        use term::dimensions;
        dimensions().map(|t| t.0)
    };
}
