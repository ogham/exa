use std::iter::repeat;
use std::string::ToString;

use colours::Colours;
use column::{Alignment, Column, Cell};
use dir::Dir;
use feature::Attribute;
use file::fields as f;
use file::File;
use options::{Columns, FileFilter, RecurseOptions, SizeFormat};

use ansi_term::{ANSIString, ANSIStrings, Style};

use datetime::local::{LocalDateTime, DatePiece};
use datetime::format::{DateFormat};
use datetime::zoned::{VariableOffset, TimeZone};

use locale;

use number_prefix::{binary_prefix, decimal_prefix, Prefixed, Standalone, PrefixNames};

use users::{OSUsers, Users};
use users::mock::MockUsers;

use super::filename;


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
#[derive(PartialEq, Debug, Copy, Clone, Default)]
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

    /// The colours to use to display information in the table, including the
    /// colour of the tree view symbols.
    pub colours: Colours,
}

impl Details {
    pub fn view(&self, dir: Option<&Dir>, files: &[File]) {
        // First, transform the Columns object into a vector of columns for
        // the current directory.
        let mut table = Table::with_options(self.colours, self.columns.for_dir(dir));
        if self.header { table.add_header() }

        // Then add files to the table and print it out.
        self.add_files_to_table(&mut table, files, 0);
        for cell in table.print_table(self.xattr, self.recurse.is_some()) {
            println!("{}", cell.text);
        }
    }

    /// Adds files to the table - recursively, if the `recurse` option
    /// is present.
    fn add_files_to_table<U: Users>(&self, table: &mut Table<U>, src: &[File], depth: usize) {
        for (index, file) in src.iter().enumerate() {
            table.add_file(file, depth, index == src.len() - 1, true);

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
    name:     Cell,

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
pub struct Table<U> {
    columns:  Vec<Column>,
    rows:     Vec<Row>,

    time:         locale::Time,
    numeric:      locale::Numeric,
    tz:           VariableOffset,
    users:        U,
    colours:      Colours,
    current_year: i64,
}

impl Default for Table<MockUsers> {
    fn default() -> Table<MockUsers> {
        Table {
            columns: Columns::default().for_dir(None),
            rows:    Vec::new(),
            time:    locale::Time::english(),
            numeric: locale::Numeric::english(),
            tz:      VariableOffset::localtime().unwrap(),
            users:   MockUsers::with_current_uid(0),
            colours: Colours::default(),
            current_year: 1234,
        }
    }
}

impl Table<OSUsers> {

    /// Create a new, empty Table object, setting the caching fields to their
    /// empty states.
    pub fn with_options(colours: Colours, columns: Vec<Column>) -> Table<OSUsers> {
        Table {
            columns: columns,
            rows:    Vec::new(),

            time:         locale::Time::load_user_locale().unwrap_or_else(|_| locale::Time::english()),
            numeric:      locale::Numeric::load_user_locale().unwrap_or_else(|_| locale::Numeric::english()),
            tz:           VariableOffset::localtime().unwrap(),
            users:        OSUsers::empty_cache(),
            colours:      colours,
            current_year: LocalDateTime::now().year(),
        }
    }
}

impl<U> Table<U> where U: Users {

    /// Add a dummy "header" row to the table, which contains the names of all
    /// the columns, underlined. This has dummy data for the cases that aren't
    /// actually used, such as the depth or list of attributes.
    pub fn add_header(&mut self) {
        let row = Row {
            depth:    0,
            cells:    self.columns.iter().map(|c| Cell::paint(self.colours.header, c.header())).collect(),
            name:     Cell::paint(self.colours.header, "Name"),
            last:     false,
            attrs:    Vec::new(),
            children: false,
        };

        self.rows.push(row);
    }

    /// Get the cells for the given file, and add the result to the table.
    pub fn add_file(&mut self, file: &File, depth: usize, last: bool, links: bool) {
        let row = Row {
            depth:    depth,
            cells:    self.cells_for_file(file),
            name:     Cell { text: filename(file, &self.colours, links), length: file.file_name_width() },
            last:     last,
            attrs:    file.xattrs.clone(),
            children: file.this.is_some(),
        };

        self.rows.push(row);
    }

    /// Use the list of columns to find which cells should be produced for
    /// this file, per-column.
    pub fn cells_for_file(&mut self, file: &File) -> Vec<Cell> {
        self.columns.clone().iter()
                    .map(|c| self.display(file, c))
                    .collect()
    }

    fn display(&mut self, file: &File, column: &Column) -> Cell {
        match *column {
            Column::Permissions    => self.render_permissions(file.permissions()),
            Column::FileSize(fmt)  => self.render_size(file.size(), fmt),
            Column::Timestamp(t)   => self.render_time(file.timestamp(t)),
            Column::HardLinks      => self.render_links(file.links()),
            Column::Inode          => self.render_inode(file.inode()),
            Column::Blocks         => self.render_blocks(file.blocks()),
            Column::User           => self.render_user(file.user()),
            Column::Group          => self.render_group(file.group()),
            Column::GitStatus      => self.render_git_status(file.git_status()),
        }
    }

    fn render_permissions(&self, permissions: f::Permissions) -> Cell {
        let c = self.colours.perms;
        let bit = |bit, chr: &'static str, style: Style| {
            if bit { style.paint(chr) } else { self.colours.punctuation.paint("-") }
        };

        let file_type = match permissions.file_type {
            f::Type::File       => self.colours.filetypes.normal.paint("."),
            f::Type::Directory  => self.colours.filetypes.directory.paint("d"),
            f::Type::Pipe       => self.colours.filetypes.special.paint("|"),
            f::Type::Link       => self.colours.filetypes.symlink.paint("l"),
            f::Type::Special    => self.colours.filetypes.special.paint("?"),
        };

        let x_colour = if let f::Type::File = permissions.file_type { c.user_execute_file }
                                                               else { c.user_execute_other };

        let mut columns = vec![
            file_type,
            bit(permissions.user_read,     "r", c.user_read),
            bit(permissions.user_write,    "w", c.user_write),
            bit(permissions.user_execute,  "x", x_colour),
            bit(permissions.group_read,    "r", c.group_read),
            bit(permissions.group_write,   "w", c.group_write),
            bit(permissions.group_execute, "x", c.group_execute),
            bit(permissions.other_read,    "r", c.other_read),
            bit(permissions.other_write,   "w", c.other_write),
            bit(permissions.other_execute, "x", c.other_execute),
        ];

        if permissions.attribute {
            columns.push(c.attribute.paint("@"));
        }

        Cell {
            text: ANSIStrings(&columns).to_string(),
            length: columns.len(),
        }
    }

    fn render_links(&self, links: f::Links) -> Cell {
        let style = if links.multiple { self.colours.links.multi_link_file }
                                 else { self.colours.links.normal };

        Cell::paint(style, &self.numeric.format_int(links.count))
    }

    fn render_blocks(&self, blocks: f::Blocks) -> Cell {
        match blocks {
            f::Blocks::Some(blocks)  => Cell::paint(self.colours.blocks, &blocks.to_string()),
            f::Blocks::None          => Cell::paint(self.colours.punctuation, "-"),
        }
    }

    fn render_inode(&self, inode: f::Inode) -> Cell {
        Cell::paint(self.colours.inode, &inode.0.to_string())
    }

    fn render_size(&self, size: f::Size, size_format: SizeFormat) -> Cell {
        if let f::Size::Some(offset) = size {
            let result = match size_format {
                SizeFormat::DecimalBytes  => decimal_prefix(offset as f64),
                SizeFormat::BinaryBytes   => binary_prefix(offset as f64),
                SizeFormat::JustBytes     => return Cell::paint(self.colours.size.numbers, &self.numeric.format_int(offset)),
            };

            match result {
                Standalone(bytes)    => Cell::paint(self.colours.size.numbers, &*bytes.to_string()),
                Prefixed(prefix, n)  => {
                    let number = if n < 10f64 { self.numeric.format_float(n, 1) } else { self.numeric.format_int(n as isize) };
                    let symbol = prefix.symbol();

                    Cell {
                        text: ANSIStrings( &[ self.colours.size.numbers.paint(&number[..]), self.colours.size.unit.paint(symbol) ]).to_string(),
                        length: number.len() + symbol.len(),
                    }
                }
            }
        }
        else {
            Cell::paint(self.colours.punctuation, "-")
        }
    }

    fn render_time(&self, timestamp: f::Time) -> Cell {
        let date = self.tz.at(LocalDateTime::at(timestamp.0));

        let format = if date.year() == self.current_year {
                DateFormat::parse("{2>:D} {:M} {2>:h}:{02>:m}").unwrap()
            }
            else {
                DateFormat::parse("{2>:D} {:M} {5>:Y}").unwrap()
            };

        Cell::paint(self.colours.date, &format.format(&date, &self.time))
    }

    fn render_git_status(&self, git: f::Git) -> Cell {
        Cell {
            text: ANSIStrings(&[ self.render_git_char(git.staged),
                                 self.render_git_char(git.unstaged) ]).to_string(),
            length: 2,
        }
    }

    fn render_git_char(&self, status: f::GitStatus) -> ANSIString {
        match status {
            f::GitStatus::NotModified  => self.colours.punctuation.paint("-"),
            f::GitStatus::New          => self.colours.git.new.paint("N"),
            f::GitStatus::Modified     => self.colours.git.modified.paint("M"),
            f::GitStatus::Deleted      => self.colours.git.deleted.paint("D"),
            f::GitStatus::Renamed      => self.colours.git.renamed.paint("R"),
            f::GitStatus::TypeChange   => self.colours.git.typechange.paint("T"),
        }
    }

    fn render_user(&mut self, user: f::User) -> Cell {
        let user_name = match self.users.get_user_by_uid(user.0) {
            Some(user)  => user.name,
            None        => user.0.to_string(),
        };

        let style = if self.users.get_current_uid() == user.0 { self.colours.users.user_you }
                                                         else { self.colours.users.user_someone_else };
        Cell::paint(style, &*user_name)
    }

    fn render_group(&mut self, group: f::Group) -> Cell {
        let mut style = self.colours.users.group_not_yours;

        let group_name = match self.users.get_group_by_gid(group.0) {
            Some(group) => {
                let current_uid = self.users.get_current_uid();
                if let Some(current_user) = self.users.get_user_by_uid(current_uid) {
                    if current_user.primary_group == group.gid || group.members.contains(&current_user.name) {
                        style = self.colours.users.group_yours;
                    }
                }
                group.name
            },
            None => group.0.to_string(),
        };

        Cell::paint(style, &*group_name)
    }

    /// Print the table to standard output, consuming it in the process.
    pub fn print_table(&self, xattr: bool, show_children: bool) -> Vec<Cell> {
        let mut stack = Vec::new();
        let mut cells = Vec::new();

        // Work out the list of column widths by finding the longest cell for
        // each column, then formatting each cell in that column to be the
        // width of that one.
        let column_widths: Vec<usize> = (0 .. self.columns.len())
            .map(|n| self.rows.iter().map(|row| row.cells[n].length).max().unwrap_or(0))
            .collect();

        for row in self.rows.iter() {
            let mut cell = Cell::empty();

            for (n, width) in column_widths.iter().enumerate() {
                match self.columns[n].alignment() {
                    Alignment::Left  => { cell.append(&row.cells[n]); cell.add_spaces(width - row.cells[n].length); }
                    Alignment::Right => { cell.add_spaces(width - row.cells[n].length); cell.append(&row.cells[n]); }
                }

                cell.add_spaces(1);
            }

            let mut filename = String::new();
            let mut filename_length = 0;

            // A stack tracks which tree characters should be printed. It's
            // necessary to maintain information about the previously-printed
            // lines, as the output will change based on whether the
            // *previous* entry was the last in its directory.
            if show_children {
                stack.resize(row.depth + 1, TreePart::Edge);
                stack[row.depth] = if row.last { TreePart::Corner } else { TreePart::Edge };

                for i in 1 .. row.depth + 1 {
                    filename.push_str(&*self.colours.punctuation.paint(stack[i].ascii_art()).to_string());
                    filename_length += 4;
                }

                if row.children {
                    stack[row.depth] = if row.last { TreePart::Blank } else { TreePart::Line };
                }

                // If any tree characters have been printed, then add an extra
                // space, which makes the output look much better.
                if row.depth != 0 {
                    filename.push(' ');
                    filename_length += 1;
                }
            }

            // Print the name without worrying about padding.
            filename.push_str(&*row.name.text);
            filename_length += row.name.length;

            if xattr {
                let width = row.attrs.iter().map(|a| a.name().len()).max().unwrap_or(0);
                for attr in row.attrs.iter() {
                    let name = attr.name();
                    let spaces: String = repeat(" ").take(width - name.len()).collect();
                    filename.push_str(&*format!("\n{}{}  {}", name, spaces, attr.size()))
                }
            }

            cell.append(&Cell { text: filename, length: filename_length });
            cells.push(cell);
        }

        cells
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
            TreePart::Edge    => "├──",
            TreePart::Line    => "│  ",
            TreePart::Corner  => "└──",
            TreePart::Blank   => "   ",
        }
    }
}


#[cfg(test)]
pub mod test {
    pub use super::Table;
    pub use file::File;
    pub use file::fields as f;

    pub use column::{Cell, Column};

    pub use users::{User, Group, uid_t, gid_t};
    pub use users::mock::MockUsers;

    pub use ansi_term::Style;
    pub use ansi_term::Colour::*;

    pub fn newser(uid: uid_t, name: &str, group: gid_t) -> User {
        User {
            uid: uid,
            name: name.to_string(),
            primary_group: group,
            home_dir: String::new(),
            shell: String::new(),
        }
    }

    // These tests create a new, default Table object, then fill in the
    // expected style in a certain way. This means we can check that the
    // right style is being used, as otherwise, it would just be plain.
    //
    // Doing things with fields is way easier than having to fake the entire
    // Metadata struct, which is what I was doing before!

    mod users {
        use super::*;

        #[test]
        fn named() {
            let mut table = Table::default();
            table.colours.users.user_you = Red.bold();

            let mut users = MockUsers::with_current_uid(1000);
            users.add_user(newser(1000, "enoch", 100));
            table.users = users;

            let user = f::User(1000);
            let expected = Cell::paint(Red.bold(), "enoch");
            assert_eq!(expected, table.render_user(user))
        }

        #[test]
        fn unnamed() {
            let mut table = Table::default();
            table.colours.users.user_you = Cyan.bold();

            let users = MockUsers::with_current_uid(1000);
            table.users = users;

            let user = f::User(1000);
            let expected = Cell::paint(Cyan.bold(), "1000");
            assert_eq!(expected, table.render_user(user));
        }

        #[test]
        fn different_named() {
            let mut table = Table::default();
            table.colours.users.user_someone_else = Green.bold();
            table.users.add_user(newser(1000, "enoch", 100));

            let user = f::User(1000);
            let expected = Cell::paint(Green.bold(), "enoch");
            assert_eq!(expected, table.render_user(user));
        }

        #[test]
        fn different_unnamed() {
            let mut table = Table::default();
            table.colours.users.user_someone_else = Red.normal();

            let user = f::User(1000);
            let expected = Cell::paint(Red.normal(), "1000");
            assert_eq!(expected, table.render_user(user));
        }

        #[test]
        fn overflow() {
            let mut table = Table::default();
            table.colours.users.user_someone_else = Blue.underline();

            let user = f::User(2_147_483_648);
            let expected = Cell::paint(Blue.underline(), "2147483648");
            assert_eq!(expected, table.render_user(user));
        }
    }

    mod groups {
        use super::*;

        #[test]
        fn named() {
            let mut table = Table::default();
            table.colours.users.group_not_yours = Fixed(101).normal();

            let mut users = MockUsers::with_current_uid(1000);
            users.add_group(Group { gid: 100, name: "folk".to_string(), members: vec![] });
            table.users = users;

            let group = f::Group(100);
            let expected = Cell::paint(Fixed(101).normal(), "folk");
            assert_eq!(expected, table.render_group(group))
        }

        #[test]
        fn unnamed() {
            let mut table = Table::default();
            table.colours.users.group_not_yours = Fixed(87).normal();

            let users = MockUsers::with_current_uid(1000);
            table.users = users;

            let group = f::Group(100);
            let expected = Cell::paint(Fixed(87).normal(), "100");
            assert_eq!(expected, table.render_group(group));
        }

        #[test]
        fn primary() {
            let mut table = Table::default();
            table.colours.users.group_yours = Fixed(64).normal();

            let mut users = MockUsers::with_current_uid(2);
            users.add_user(newser(2, "eve", 100));
            users.add_group(Group { gid: 100, name: "folk".to_string(), members: vec![] });
            table.users = users;

            let group = f::Group(100);
            let expected = Cell::paint(Fixed(64).normal(), "folk");
            assert_eq!(expected, table.render_group(group))
        }

        #[test]
        fn secondary() {
            let mut table = Table::default();
            table.colours.users.group_yours = Fixed(31).normal();

            let mut users = MockUsers::with_current_uid(2);
            users.add_user(newser(2, "eve", 666));
            users.add_group(Group { gid: 100, name: "folk".to_string(), members: vec![ "eve".to_string() ] });
            table.users = users;

            let group = f::Group(100);
            let expected = Cell::paint(Fixed(31).normal(), "folk");
            assert_eq!(expected, table.render_group(group))
        }

        #[test]
        fn overflow() {
            let mut table = Table::default();
            table.colours.users.group_not_yours = Blue.underline();

            let group = f::Group(2_147_483_648);
            let expected = Cell::paint(Blue.underline(), "2147483648");
            assert_eq!(expected, table.render_group(group));
        }
    }
}
