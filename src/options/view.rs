use std::env::var_os;

use getopts;

use output::Colours;
use output::{Grid, Details, GridDetails, Lines};
use options::{FileFilter, DirAction, Misfire};
use output::column::{Columns, TimeTypes, SizeFormat};
use term::dimensions;
use fs::feature::xattr;


/// The **view** contains all information about how to format output.
#[derive(PartialEq, Debug, Clone)]
pub enum View {
    Details(Details),
    Grid(Grid),
    GridDetails(GridDetails),
    Lines(Lines),
}

impl View {

    /// Determine which view to use and all of that view’s arguments.
    pub fn deduce(matches: &getopts::Matches, filter: FileFilter, dir_action: DirAction) -> Result<View, Misfire> {
        use options::misfire::Misfire::*;

        let long = || {
            if matches.opt_present("across") && !matches.opt_present("grid") {
                Err(Useless("across", true, "long"))
            }
            else if matches.opt_present("oneline") {
                Err(Useless("oneline", true, "long"))
            }
            else {
                let term_colours = try!(TerminalColours::deduce(matches));
                let colours = match term_colours {
                    TerminalColours::Always    => Colours::colourful(),
                    TerminalColours::Never     => Colours::plain(),
                    TerminalColours::Automatic => {
                        if dimensions().is_some() {
                            Colours::colourful()
                        }
                        else {
                            Colours::plain()
                        }
                    },
                };

                let details = Details {
                    columns: Some(try!(Columns::deduce(matches))),
                    header: matches.opt_present("header"),
                    recurse: dir_action.recurse_options(),
                    filter: filter.clone(),
                    xattr: xattr::ENABLED && matches.opt_present("extended"),
                    colours: colours,
                };

                Ok(details)
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
            let term_colours = try!(TerminalColours::deduce(matches));
            let term_width   = try!(TerminalWidth::deduce());

            if let Some(&width) = term_width.as_ref() {
                let colours = match term_colours {
                    TerminalColours::Always    => Colours::colourful(),
                    TerminalColours::Never     => Colours::plain(),
                    TerminalColours::Automatic => Colours::colourful(),
                };

                if matches.opt_present("oneline") {
                    if matches.opt_present("across") {
                        Err(Useless("across", true, "oneline"))
                    }
                    else {
                        let lines = Lines {
                             colours: colours,
                        };

                        Ok(View::Lines(lines))
                    }
                }
                else if matches.opt_present("tree") {
                    let details = Details {
                        columns: None,
                        header: false,
                        recurse: dir_action.recurse_options(),
                        filter: filter.clone(),  // TODO: clone
                        xattr: false,
                        colours: colours,
                    };

                    Ok(View::Details(details))
                }
                else {
                    let grid = Grid {
                        across: matches.opt_present("across"),
                        console_width: width,
                        colours: colours,
                    };

                    Ok(View::Grid(grid))
                }
            }
            else {
                // If the terminal width couldn’t be matched for some reason, such
                // as the program’s stdout being connected to a file, then
                // fallback to the lines view.

                let colours = match term_colours {
                    TerminalColours::Always    => Colours::colourful(),
                    TerminalColours::Never     => Colours::plain(),
                    TerminalColours::Automatic => Colours::plain(),
                };

                if matches.opt_present("tree") {
                    let details = Details {
                        columns: None,
                        header: false,
                        recurse: dir_action.recurse_options(),
                        filter: filter.clone(),
                        xattr: false,
                        colours: colours,
                    };

                    Ok(View::Details(details))
                }
                else {
                    let lines = Lines {
                         colours: colours,
                    };

                    Ok(View::Lines(lines))
                }
            }
        };

        if matches.opt_present("long") {
            let long_options = try!(long());

            if matches.opt_present("grid") {
                match other_options_scan() {
                    Ok(View::Grid(grid)) => return Ok(View::GridDetails(GridDetails { grid: grid, details: long_options })),
                    Ok(lines)            => return Ok(lines),
                    Err(e)               => return Err(e),
                };
            }
            else {
                return Ok(View::Details(long_options));
            }
        }

        try!(long_options_scan());

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
        else if let Some((width, _)) = dimensions() {
            Ok(TerminalWidth::Terminal(width))
        }
        else {
            Ok(TerminalWidth::Unset)
        }
    }

    fn as_ref(&self) -> Option<&usize> {
        match *self {
            TerminalWidth::Set(ref width)       => Some(width),
            TerminalWidth::Terminal(ref width)  => Some(width),
            TerminalWidth::Unset                => None,
        }
    }
}


impl Columns {
    fn deduce(matches: &getopts::Matches) -> Result<Columns, Misfire> {
        Ok(Columns {
            size_format: try!(SizeFormat::deduce(matches)),
            time_types:  try!(TimeTypes::deduce(matches)),
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

            match &*word {
                "mod" | "modified"  => Ok(TimeTypes { accessed: false, modified: true,  created: false }),
                "acc" | "accessed"  => Ok(TimeTypes { accessed: true,  modified: false, created: false }),
                "cr"  | "created"   => Ok(TimeTypes { accessed: false, modified: false, created: true  }),
                otherwise           => Err(Misfire::bad_argument("time", otherwise,
                                                                 &["modified", "accessed", "created"])),
            }
        }
        else if modified || created || accessed {
            Ok(TimeTypes { accessed: accessed, modified: modified, created: created })
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
        if let Some(word) = matches.opt_str("color").or(matches.opt_str("colour")) {
            match &*word {
                "always"              => Ok(TerminalColours::Always),
                "auto" | "automatic"  => Ok(TerminalColours::Automatic),
                "never"               => Ok(TerminalColours::Never),
                otherwise             => Err(Misfire::bad_argument("color", otherwise,
                                                                   &["always", "auto", "never"]))
            }
        }
        else {
            Ok(TerminalColours::default())
        }
    }
}
