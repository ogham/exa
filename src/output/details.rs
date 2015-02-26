use column::{Alignment, Column, Cell};
use xattr::Attribute;
use dir::Dir;
use file::{File, GREY};
use options::{Columns, FileFilter, RecurseOptions};
use users::OSUsers;

use locale;
use ansi_term::Style::Plain;

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
#[derive(PartialEq, Debug, Copy)]
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
}

impl Details {
    pub fn view(&self, dir: Option<&Dir>, files: &[File]) {
        // First, transform the Columns object into a vector of columns for
        // the current directory.
        let mut table = Table::with_columns(self.columns.for_dir(dir));
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
    columns: Vec<Column>,
    users:   OSUsers,
    locale:  UserLocale,
    rows:    Vec<Row>,
}

impl Table {
    /// Create a new, empty Table object, setting the caching fields to their
    /// empty states.
    fn with_columns(columns: Vec<Column>) -> Table {
        Table {
            columns: columns,
            users: OSUsers::empty_cache(),
            locale: UserLocale::new(),
            rows: Vec::new(),
        }
    }

    /// Add a dummy "header" row to the table, which contains the names of all
    /// the columns, underlined. This has dummy data for the cases that aren't
    /// actually used, such as the depth or list of attributes.
    fn add_header(&mut self) {
        let row = Row {
            depth:    0,
            cells:    self.columns.iter().map(|c| Cell::paint(Plain.underline(), c.header())).collect(),
            name:     Plain.underline().paint("Name").to_string(),
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
                    .map(|c| file.display(c, &mut self.users, &self.locale))
                    .collect()
    }

    /// Get the cells for the given file, and add the result to the table.
    fn add_file(&mut self, file: &File, depth: usize, last: bool) {
        let row = Row {
            depth:    depth,
            cells:    self.cells_for_file(file),
            name:     file.file_name_view(),
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
        let column_widths: Vec<usize> = range(0, self.columns.len())
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
                    print!("{}", GREY.paint(stack[i].ascii_art()));
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

pub struct UserLocale {
    pub time:    locale::Time,
    pub numeric: locale::Numeric,
}

impl UserLocale {
    pub fn new() -> UserLocale {
        UserLocale {
            time:    locale::Time::load_user_locale().unwrap_or_else(|_| locale::Time::english()),
            numeric: locale::Numeric::load_user_locale().unwrap_or_else(|_| locale::Numeric::english()),
        }
    }

    pub fn default() -> UserLocale {
        UserLocale {
            time:    locale::Time::english(),
            numeric: locale::Numeric::english(),
        }
    }
}
