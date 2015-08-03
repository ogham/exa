use ansi_term::ANSIStrings;

use colours::Colours;
use file::File;
use filetype::file_colour;

pub use self::details::Details;
pub use self::grid::Grid;
pub use self::lines::Lines;
pub use self::grid_details::GridDetails;

mod grid;
pub mod details;
mod lines;
mod grid_details;

pub fn filename(file: &File, colours: &Colours, links: bool) -> String {
    if links && file.is_link() {
        symlink_filename(file, colours)
    }
    else {
        let style = file_colour(colours, file);
        style.paint(&file.name).to_string()
    }
}

fn symlink_filename(file: &File, colours: &Colours) -> String {
    match file.link_target() {
        Ok(target) => format!("{} {} {}",
                              file_colour(colours, file).paint(&file.name),
                              colours.punctuation.paint("->"),
                              ANSIStrings(&[ colours.symlink_path.paint(&target.path_prefix()),
                                             file_colour(colours, &target).paint(&target.name) ])),

        Err(filename) => format!("{} {} {}",
                                 file_colour(colours, file).paint(&file.name),
                                 colours.broken_arrow.paint("->"),
                                 colours.broken_filename.paint(&filename)),
    }
}
