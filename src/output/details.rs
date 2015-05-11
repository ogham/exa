use colours::Colours;
use column::{Alignment, Column, Cell};
use feature::Attribute;
use dir::Dir;
use file::{Blocks, File, Git, GitStatus, Group, Inode, Links, Permissions, Size, Time, Type, User};
use options::{Columns, FileFilter, RecurseOptions, SizeFormat};
use users::{OSUsers, Users};

use super::filename;

use ansi_term::{ANSIString, ANSIStrings, Style};
use ansi_term::Style::Plain;
use ansi_term::Colour::Fixed;

use locale;

use number_prefix::{binary_prefix, decimal_prefix, Prefixed, Standalone, PrefixNames};

use datetime::local::{LocalDateTime, DatePiece};
use datetime::format::{DateFormat};

/// With the **Details** view, the output gets formatted into columns, with
/// each `Column` object showing some piece of information about the file,
/// such as its size, or its permissions.
///
/// To do this, the results have to be written to a table, instead of
/// displaying each file immediately. Then, the width of each column can be
/// calculated based on the individual results, and the fields are padded
/// during output.
///
/// Almost all the heavy lifting is done in a Table object, which handles the
/// columns for each row.
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Details {

    /// A Columns object that says which columns should be included in the
    /// output in the general case. Directories themselves can pick which
    /// columns are *added* to this list, such as the Git column.
    pub columns: Columns,

    /// Whether to recurse through directories with a tree view, and if so,
    /// which options to use. This field is only relevant here if the `tree`
    /// field of the RecurseOptions is `true`.
    pub recurse: Option<(RecurseOptions, FileFilter)>,

    /// Whether to show a header line or not.
    pub header: bool,

    /// Whether to show each file's extended attributes.
    pub xattr: bool,

    pub colours: Colours,
}

impl Details {
    pub fn view(&self, dir: Option<&Dir>, files: &[File]) {
        // First, transform the Columns object into a vector of columns for
        // the current directory.
        let mut table = Table::with_options(self.colours, self.columns.for_dir(dir), self.columns.size_format);
        if self.header { table.add_header() }

        // Then add files to the table and print it out.
        self.add_files_to_table(&mut table, files, 0);
        table.print_table(self.xattr, self.recurse.is_some());
    }

    /// Adds files to the table - recursively, if the `recurse` option
    /// is present.
    fn add_files_to_table(&self, table: &mut Table, src: &[File], depth: usize) {
        for (index, file) in src.iter().enumerate() {
            table.add_file(file, depth, index == src.len() - 1);

            // There are two types of recursion that exa supports: a tree
            // view, which is dealt with here, and multiple listings, which is
            // dealt with in the main module. So only actually recurse if we
            // are in tree mode - the other case will be dealt with elsewhere.
            if let Some((r, filter)) = self.recurse {
                if r.tree == false || r.is_too_deep(depth) {
                    continue;
                }

                // Use the filter to remove unwanted files *before* expanding
                // them, so we don't examine any directories that wouldn't
                // have their contents listed anyway.
                if let Some(ref dir) = file.this {
                    let mut files = dir.files(true);
                    filter.transform_files(&mut files);
                    self.add_files_to_table(table, &files, depth + 1);
                }
            }
        }
    }
}

struct Row {

    /// Vector of cells to display.
    cells:    Vec<Cell>,

    /// This file's name, in coloured output. The name is treated separately
    /// from the other cells, as it never requires padding.
    name:     String,

    /// How many directories deep into the tree structure this is. Directories
    /// on top have depth 0.
    depth:    usize,

    /// Vector of this file's extended attributes, if that feature is active.
    attrs:    Vec<Attribute>,

    /// Whether this is the last entry in the directory. This flag is used
    /// when calculating the tree view.
    last:     bool,

    /// Whether this file is a directory and has any children. Also used when
    /// calculating the tree view.
    children: bool,
}

/// A **Table** object gets built up by the view as it lists files and
/// directories.
struct Table {
    columns:  Vec<Column>,
    rows:     Vec<Row>,

    local:    Locals,
    colours:  Colours,
}

impl Table {
    /// Create a new, empty Table object, setting the caching fields to their
    /// empty states.
    fn with_options(colours: Colours, columns: Vec<Column>, size: SizeFormat) -> Table {
        Table {
            columns: columns,
            local: Locals {
                time:         locale::Time::load_user_locale().unwrap_or_else(|_| locale::Time::english()),
                numeric:      locale::Numeric::load_user_locale().unwrap_or_else(|_| locale::Numeric::english()),
                users:        OSUsers::empty_cache(),
                current_year: LocalDateTime::now().year(),
                size_format:  size,
            },
            rows: Vec::new(),
            colours: colours,
        }
    }

    /// Add a dummy "header" row to the table, which contains the names of all
    /// the columns, underlined. This has dummy data for the cases that aren't
    /// actually used, such as the depth or list of attributes.
    fn add_header(&mut self) {
        let row = Row {
            depth:    0,
            cells:    self.columns.iter().map(|c| Cell::paint(self.colours.header, c.header())).collect(),
            name:     self.colours.header.paint("Name").to_string(),
            last:     false,
            attrs:    Vec::new(),
            children: false,
        };

        self.rows.push(row);
    }

    /// Use the list of columns to find which cells should be produced for
    /// this file, per-column.
    fn cells_for_file(&mut self, file: &File) -> Vec<Cell> {
        self.columns.clone().iter()
                    .map(|c| self.display(file, c))
                    .collect()
    }

    fn display(&mut self, file: &File, column: &Column) -> Cell {
        match *column {
            Column::Permissions     => file.permissions().render(&self.colours, &mut self.local),
            Column::FileSize(f)     => file.size().render(&self.colours, &mut self.local),
            Column::Timestamp(t, y) => file.timestamp(t).render(&self.colours, &mut self.local),
            Column::HardLinks       => file.links().render(&self.colours, &mut self.local),
            Column::Inode           => file.inode().render(&self.colours, &mut self.local),
            Column::Blocks          => file.blocks().render(&self.colours, &mut self.local),
            Column::User            => file.user().render(&self.colours, &mut self.local),
            Column::Group           => file.group().render(&self.colours, &mut self.local),
            Column::GitStatus       => file.git_status().render(&self.colours, &mut self.local),
        }
    }

    /// Get the cells for the given file, and add the result to the table.
    fn add_file(&mut self, file: &File, depth: usize, last: bool) {
        let row = Row {
            depth:    depth,
            cells:    self.cells_for_file(file),
            name:     filename(file, &self.colours),
            last:     last,
            attrs:    file.xattrs.clone(),
            children: file.this.is_some(),
        };

        self.rows.push(row)
    }

    /// Print the table to standard output, consuming it in the process.
    fn print_table(self, xattr: bool, show_children: bool) {
        let mut stack = Vec::new();

        // Work out the list of column widths by finding the longest cell for
        // each column, then formatting each cell in that column to be the
        // width of that one.
        let column_widths: Vec<usize> = (0 .. self.columns.len())
            .map(|n| self.rows.iter().map(|row| row.cells[n].length).max().unwrap_or(0))
            .collect();

        for row in self.rows.into_iter() {
            for (n, width) in column_widths.iter().enumerate() {
                let padding = width - row.cells[n].length;
                print!("{} ", self.columns[n].alignment().pad_string(&row.cells[n].text, padding));
            }

            // A stack tracks which tree characters should be printed. It's
            // necessary to maintain information about the previously-printed
            // lines, as the output will change based on whether the
            // *previous* entry was the last in its directory.
            if show_children {
                stack.resize(row.depth + 1, TreePart::Edge);
                stack[row.depth] = if row.last { TreePart::Corner } else { TreePart::Edge };

                for i in 1 .. row.depth + 1 {
                    print!("{}", self.colours.punctuation.paint(stack[i].ascii_art()));
                }

                if row.children {
                    stack[row.depth] = if row.last { TreePart::Blank } else { TreePart::Line };
                }

                // If any tree characters have been printed, then add an extra
                // space, which makes the output look much better.
                if row.depth != 0 {
                    print!(" ");
                }
            }

            // Print the name without worrying about padding.
            print!("{}\n", row.name);

            if xattr {
                let width = row.attrs.iter().map(|a| a.name().len()).max().unwrap_or(0);
                for attr in row.attrs.iter() {
                    let name = attr.name();
                    println!("{}\t{}",
                        Alignment::Left.pad_string(name, width - name.len()),
                        attr.size()
                    )
                }
            }
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
enum TreePart {

    /// Rightmost column, *not* the last in the directory.
    Edge,

    /// Not the rightmost column, and the directory has not finished yet.
    Line,

    /// Rightmost column, and the last in the directory.
    Corner,

    /// Not the rightmost column, and the directory *has* finished.
    Blank,
}

impl TreePart {
    fn ascii_art(&self) -> &'static str {
        match *self {
            TreePart::Edge   => "├──",
            TreePart::Line   => "│  ",
            TreePart::Corner => "└──",
            TreePart::Blank  => "   ",
        }
    }
}

pub trait Render {
    fn render(self, colours: &Colours, local: &mut Locals) -> Cell;
}

impl Render for Permissions {
    fn render(self, colours: &Colours, local: &mut Locals) -> Cell {
        let c = colours.perms;
        let bit = |bit, chr: &'static str, style: Style| {
            if bit { style.paint(chr) } else { colours.punctuation.paint("-") }
        };

        let file_type = match self.file_type {
            Type::File      => colours.filetypes.normal.paint("."),
            Type::Directory => colours.filetypes.directory.paint("d"),
            Type::Pipe      => colours.filetypes.special.paint("|"),
            Type::Link      => colours.filetypes.symlink.paint("l"),
            Type::Special   => colours.filetypes.special.paint("?"),
        };

        let x_colour = if let Type::File = self.file_type { c.user_execute_file }
                                                     else { c.user_execute_other};

        let string = ANSIStrings( &[
            file_type,
            bit(self.user_read,     "r", c.user_read),
            bit(self.user_write,    "w", c.user_write),
            bit(self.user_execute,  "x", x_colour),
            bit(self.group_read,    "r", c.group_read),
            bit(self.group_write,   "w", c.group_write),
            bit(self.group_execute, "x", c.group_execute),
            bit(self.other_read,    "r", c.other_read),
            bit(self.other_write,   "w", c.other_write),
            bit(self.other_execute, "x", c.other_execute),
            if self.attribute { c.attribute.paint("@") } else { Plain.paint(" ") },
        ]).to_string();

        Cell {
            text: string,
            length: 11,
        }
    }
}

impl Render for Links {
    fn render(self, colours: &Colours, local: &mut Locals) -> Cell {
        let style = if self.multiple { colours.links.multi_link_file }
                                else { colours.links.normal };
        Cell::paint(style, &local.numeric.format_int(self.count))
    }
}

impl Render for Blocks {
    fn render(self, colours: &Colours, local: &mut Locals) -> Cell {
        match self {
            Blocks::Some(blocks) => Cell::paint(colours.blocks, &blocks.to_string()),
            Blocks::None         => Cell::paint(colours.punctuation, "-"),
        }
    }
}

impl Render for Inode {
    fn render(self, colours: &Colours, local: &mut Locals) -> Cell {
        Cell::paint(colours.inode, &self.0.to_string())
    }
}

impl Render for Size {
    fn render(self, colours: &Colours, local: &mut Locals) -> Cell {
        if let Size::Some(offset) = self {
            let result = match local.size_format {
                SizeFormat::DecimalBytes => decimal_prefix(offset as f64),
                SizeFormat::BinaryBytes  => binary_prefix(offset as f64),
                SizeFormat::JustBytes    => return Cell::paint(colours.size.numbers, &local.numeric.format_int(offset)),
            };

            match result {
                Standalone(bytes) => Cell::paint(colours.size.numbers, &*bytes.to_string()),
                Prefixed(prefix, n) => {
                    let number = if n < 10f64 { local.numeric.format_float(n, 1) } else { local.numeric.format_int(n as isize) };
                    let symbol = prefix.symbol();

                    Cell {
                        text: ANSIStrings( &[ colours.size.numbers.paint(&number[..]), colours.size.unit.paint(symbol) ]).to_string(),
                        length: number.len() + symbol.len(),
                    }
                }
            }
        }
        else {
            Cell::paint(colours.punctuation, "-")
        }
    }
}

impl Render for Time {
    fn render(self, colours: &Colours, local: &mut Locals) -> Cell {
        let date = LocalDateTime::at(self.0);

        let format = if date.year() == local.current_year {
                DateFormat::parse("{2>:D} {:M} {2>:h}:{02>:m}").unwrap()
            }
            else {
                DateFormat::parse("{2>:D} {:M} {5>:Y}").unwrap()
            };

        Cell::paint(colours.date, &format.format(date, &local.time))
    }
}

impl Render for Git {
    fn render(self, colours: &Colours, local: &mut Locals) -> Cell {
        let render_char = |chr| {
            match chr {
                GitStatus::NotModified => colours.punctuation.paint("-"),
                GitStatus::New         => colours.git.renamed.paint("N"),
                GitStatus::Modified    => colours.git.renamed.paint("M"),
                GitStatus::Deleted     => colours.git.renamed.paint("D"),
                GitStatus::Renamed     => colours.git.renamed.paint("R"),
                GitStatus::TypeChange  => colours.git.renamed.paint("T"),
            }
        };

        Cell {
            text: ANSIStrings(&[ render_char(self.staged), render_char(self.unstaged) ]).to_string(),
            length: 2,
        }
    }
}

impl Render for User {
    fn render(self, colours: &Colours, local: &mut Locals) -> Cell {
        let user_name = match local.users.get_user_by_uid(self.0) {
            Some(user) => user.name,
            None => self.0.to_string(),
        };

        let style = if local.users.get_current_uid() == self.0 { colours.users.user_you }
                                                          else { colours.users.user_someone_else };
        Cell::paint(style, &*user_name)
    }
}

impl Render for Group {
    fn render(self, colours: &Colours, local: &mut Locals) -> Cell {
        let mut style = colours.users.group_not_yours;

        let group_name = match local.users.get_group_by_gid(self.0) {
            Some(group) => {
                let current_uid = local.users.get_current_uid();
                if let Some(current_user) = local.users.get_user_by_uid(current_uid) {
                    if current_user.primary_group == group.gid || group.members.contains(&current_user.name) {
                        style = colours.users.group_yours;
                    }
                }
                group.name
            },
            None => self.0.to_string(),
        };

        Cell::paint(style, &*group_name)
    }
}

pub struct Locals {
    pub time:    locale::Time,
    pub numeric: locale::Numeric,
    pub users:   OSUsers,
    pub size_format: SizeFormat,
    pub current_year: i64,
}
