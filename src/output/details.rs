use column::{Column, Cell};
use dir::Dir;
use file::{File, GREY};
use options::{Columns, FileFilter};
use users::OSUsers;

use ansi_term::Style::Plain;

#[derive(PartialEq, Debug, Copy)]
pub struct Details {
    pub columns: Columns,
    pub header: bool,
    pub tree: bool,
    pub filter: FileFilter,
}

impl Details {

    pub fn view(&self, dir: Option<&Dir>, files: &[File]) {
        // The output gets formatted into columns, which looks nicer. To
        // do this, we have to write the results into a table, instead of
        // displaying each file immediately, then calculating the maximum
        // width of each column based on the length of the results and
        // padding the fields during output.

        let columns = self.columns.for_dir(dir);
        let mut cache = OSUsers::empty_cache();
        let mut table = Vec::new();
        self.get_files(&columns[], &mut cache, &mut table, files, 0);

        if self.header {
            let row = Row {
                depth: 0,
                cells: columns.iter().map(|c| Cell::paint(Plain.underline(), c.header())).collect(),
                name: Plain.underline().paint("Name").to_string(),
                last: false,
                children: false,
            };

            table.insert(0, row);
        }

        let column_widths: Vec<usize> = range(0, columns.len())
            .map(|n| table.iter().map(|row| row.cells[n].length).max().unwrap_or(0))
            .collect();

        let mut stack = Vec::new();

        for row in table {
            for (num, column) in columns.iter().enumerate() {
                let padding = column_widths[num] - row.cells[num].length;
                print!("{} ", column.alignment().pad_string(&row.cells[num].text, padding));
            }

            if self.tree {
                stack.resize(row.depth  + 1, "├──");
                stack[row.depth] = if row.last { "└──" } else { "├──" };

                for i in 1 .. row.depth + 1 {
                    print!("{}", GREY.paint(stack[i]));
                }

                if row.children {
                    stack[row.depth] = if row.last { "   " } else { "│  " };
                }

                if row.depth != 0 {
                    print!(" ");
                }
            }

            print!("{}\n", row.name);
        }
    }

    fn get_files(&self, columns: &[Column], cache: &mut OSUsers, dest: &mut Vec<Row>, src: &[File], depth: usize) {
        for (index, file) in src.iter().enumerate() {

            let row = Row {
                depth: depth,
                cells: columns.iter().map(|c| file.display(c, cache)).collect(),
                name:  file.file_name_view(),
                last:  index == src.len() - 1,
                children: file.this.is_some(),
            };

            dest.push(row);

            if self.tree {
                if let Some(ref dir) = file.this {
                    let mut files = dir.files(true);
                    self.filter.transform_files(&mut files);
                    self.get_files(columns, cache, dest, files.as_slice(), depth + 1);
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
    pub children: bool,
}
