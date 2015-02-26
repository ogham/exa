use column::{Alignment, Column, Cell};
use xattr::Attribute;
use dir::Dir;
use file::{File, GREY};
use options::{Columns, FileFilter, RecurseOptions};
use users::OSUsers;

use locale;
use ansi_term::Style::Plain;

#[derive(PartialEq, Debug, Copy)]
pub struct Details {
    pub columns: Columns,
    pub header: bool,
    pub recurse: Option<RecurseOptions>,
    pub xattr: bool,
    pub filter: FileFilter,
}

impl Details {
    pub fn view(&self, dir: Option<&Dir>, files: &[File]) {
        // The output gets formatted into columns, which looks nicer. To
        // do this, we have to write the results into a table, instead of
        // displaying each file immediately, then calculating the maximum
        // width of each column based on the length of the results and
        // padding the fields during output.

        // Almost all the heavy lifting is done in a Table object, which
        // automatically calculates the width of each column and the
        // appropriate padding.
        let mut table = Table::with_columns(self.columns.for_dir(dir));
        if self.header { table.add_header() }

        self.add_files_to_table(&mut table, files, 0);
        table.print_table(self.xattr, self.recurse.is_some());
    }

    /// Adds files to the table - recursively, if the `recurse` option
    /// is present.
    fn add_files_to_table(&self, table: &mut Table, src: &[File], depth: usize) {
        for (index, file) in src.iter().enumerate() {
            table.add_row(file, depth, index == src.len() - 1);

            if let Some(r) = self.recurse {
                if r.tree == false || r.is_too_deep(depth) {
                    continue;
                }

                if let Some(ref dir) = file.this {
                    let mut files = dir.files(true);
                    self.filter.transform_files(&mut files);
                    self.add_files_to_table(table, &files, depth + 1);
                }
            }
        }
    }
}

struct Row {
    pub depth: usize,
    pub cells: Vec<Cell>,
    pub name: String,
    pub last: bool,
    pub attrs: Vec<Attribute>,
    pub children: bool,
}

type ColumnInfo = (usize, Alignment);

struct Table {
    columns: Vec<Column>,
    users: OSUsers,
    locale: UserLocale,
    rows: Vec<Row>,
}

impl Table {
    fn with_columns(columns: Vec<Column>) -> Table {
        Table {
            columns: columns,
            users: OSUsers::empty_cache(),
            locale: UserLocale::new(),
            rows: Vec::new(),
        }
    }

    fn add_header(&mut self) {
        let row = Row {
            depth: 0,
            cells: self.columns.iter().map(|c| Cell::paint(Plain.underline(), c.header())).collect(),
            name: Plain.underline().paint("Name").to_string(),
            last: false,
            attrs: Vec::new(),
            children: false,
        };

        self.rows.push(row);
    }

    fn cells_for_file(&mut self, file: &File) -> Vec<Cell> {
        self.columns.clone().iter()
                    .map(|c| file.display(c, &mut self.users, &self.locale))
                    .collect()
    }

    fn add_row(&mut self, file: &File, depth: usize, last: bool) {
        let row = Row {
            depth: depth,
            cells: self.cells_for_file(file),
            name: file.file_name_view(),
            last: last,
            attrs: file.xattrs.clone(),
            children: file.this.is_some(),
        };

        self.rows.push(row)
    }

    fn print_table(self, xattr: bool, show_children: bool) {
        let mut stack = Vec::new();

        let column_widths: Vec<usize> = range(0, self.columns.len())
            .map(|n| self.rows.iter().map(|row| row.cells[n].length).max().unwrap_or(0))
            .collect();

        for row in self.rows.iter() {
            for (n, width) in column_widths.iter().enumerate() {
                let padding = width - row.cells[n].length;
                print!("{} ", self.columns[n].alignment().pad_string(&row.cells[n].text, padding));
            }

            if show_children {
                stack.resize(row.depth + 1, TreePart::Edge);
                stack[row.depth] = if row.last { TreePart::Corner } else { TreePart::Edge };

                for i in 1 .. row.depth + 1 {
                    print!("{}", GREY.paint(stack[i].ascii_art()));
                }

                if row.children {
                    stack[row.depth] = if row.last { TreePart::Blank } else { TreePart::Line };
                }

                if row.depth != 0 {
                    print!(" ");
                }
            }

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
    Edge,
    Corner,
    Blank,
    Line,
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
    pub time: locale::Time,
    pub numeric: locale::Numeric,
}

impl UserLocale {
    pub fn new() -> UserLocale {
        UserLocale {
            time: locale::Time::load_user_locale().unwrap_or_else(|_| locale::Time::english()),
            numeric: locale::Numeric::load_user_locale().unwrap_or_else(|_| locale::Numeric::english()),
        }
    }

    pub fn default() -> UserLocale {
        UserLocale {
            time: locale::Time::english(),
            numeric: locale::Numeric::english(),
        }
    }
}
