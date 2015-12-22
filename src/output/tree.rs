//! Tree structures, such as `├──` or `└──`, used in a tree view.
//!
//! ## Constructing Tree Views
//!
//! When using the `--tree` argument, instead of a vector of cells, each row
//! has a `depth` field that indicates how far deep in the tree it is: the top
//! level has depth 0, its children have depth 1, and *their* children have
//! depth 2, and so on.
//!
//! On top of this, it also has a `last` field that specifies whether this is
//! the last row of this particular consecutive set of rows. This doesn’t
//! affect the file’s information; it’s just used to display a different set of
//! Unicode tree characters! The resulting table looks like this:
//!
//!     ┌───────┬───────┬───────────────────────┐
//!     │ Depth │ Last  │ Output                │
//!     ├───────┼───────┼───────────────────────┤
//!     │     0 │       │ documents             │
//!     │     1 │ false │ ├── this_file.txt     │
//!     │     1 │ false │ ├── that_file.txt     │
//!     │     1 │ false │ ├── features          │
//!     │     2 │ false │ │  ├── feature_1.rs   │
//!     │     2 │ false │ │  ├── feature_2.rs   │
//!     │     2 │ true  │ │  └── feature_3.rs   │
//!     │     1 │ true  │ └── pictures          │
//!     │     2 │ false │    ├── garden.jpg     │
//!     │     2 │ false │    ├── flowers.jpg    │
//!     │     2 │ false │    ├── library.png    │
//!     │     2 │ true  │    └── space.tiff     │
//!     └───────┴───────┴───────────────────────┘
//!
//! Creating the table like this means that each file has to be tested to see
//! if it’s the last one in the group. This is usually done by putting all the
//! files in a vector beforehand, getting its length, then comparing the index
//! of each file to see if it’s the last one. (As some files may not be
//! successfully `stat`ted, we don’t know how many files are going to exist in
//! each directory)

#[derive(PartialEq, Debug, Clone)]
pub enum TreePart {

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

    /// Turn this tree part into ASCII-licious box drawing characters!
    /// (Warning: not actually ASCII)
    pub fn ascii_art(&self) -> &'static str {
        match *self {
            TreePart::Edge    => "├──",
            TreePart::Line    => "│  ",
            TreePart::Corner  => "└──",
            TreePart::Blank   => "   ",
        }
    }
}


/// A **tree trunk** builds up arrays of tree parts over multiple depths.
#[derive(Debug, Default)]
pub struct TreeTrunk {

    /// A stack tracks which tree characters should be printed. It’s
    /// necessary to maintain information about the previously-printed
    /// lines, as the output will change based on any previous entries.
    stack: Vec<TreePart>,

    /// A tuple for the last ‘depth’ and ‘last’ parameters that are passed in.
    last_params: Option<(usize, bool)>,
}

impl TreeTrunk {

    /// Calculates the tree parts for an entry at the given depth and
    /// last-ness. The depth is used to determine where in the stack the tree
    /// part should be inserted, and the last-ness is used to determine which
    /// type of tree part to insert.
    ///
    /// This takes a `&mut self` because the results of each file are stored
    /// and used in future rows.
    pub fn new_row(&mut self, depth: usize, last: bool) -> &[TreePart] {

        // If this isn’t our first iteration, then update the tree parts thus
        // far to account for there being another row after it.
        if let Some((last_depth, last_last)) = self.last_params {
            self.stack[last_depth] = if last_last { TreePart::Blank } else { TreePart::Line };
        }

        // Make sure the stack has enough space, then add or modify another
        // part into it.
        self.stack.resize(depth + 1, TreePart::Edge);
        self.stack[depth] = if last { TreePart::Corner } else { TreePart::Edge };
        self.last_params = Some((depth, last));

        // Return the tree parts as a slice of the stack.
        //
        // Ignoring the first component is specific to exa: when a user prints
        // a tree view for multiple directories, we don’t want there to be a
        // ‘zeroth level’ connecting the initial directories. Otherwise, not
        // only are unrelated directories seemingly connected to each other,
        // but the tree part of the first row doesn’t connect to anything:
        //
        // with [0..]             with [1..]
        // ==========             ==========
        // ├──folder              folder
        // │  └──file             └──file
        // └──folder              folder
        //    └──file             └──file
        &self.stack[1..]
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_at_first() {
        let mut tt = TreeTrunk::default();
        assert_eq!(tt.new_row(0, true), &[]);
    }

    #[test]
    fn one_child() {
        let mut tt = TreeTrunk::default();
        assert_eq!(tt.new_row(0, true), &[]);
        assert_eq!(tt.new_row(1, true), &[ TreePart::Corner ]);
    }

    #[test]
    fn two_children() {
        let mut tt = TreeTrunk::default();
        assert_eq!(tt.new_row(0, true), &[]);
        assert_eq!(tt.new_row(1, false), &[ TreePart::Edge ]);
        assert_eq!(tt.new_row(1, true),  &[ TreePart::Corner ]);
    }

    #[test]
    fn two_times_two_children() {
        let mut tt = TreeTrunk::default();
        assert_eq!(tt.new_row(0, false), &[]);
        assert_eq!(tt.new_row(1, false), &[ TreePart::Edge ]);
        assert_eq!(tt.new_row(1, true),  &[ TreePart::Corner ]);

        assert_eq!(tt.new_row(0, true), &[]);
        assert_eq!(tt.new_row(1, false), &[ TreePart::Edge ]);
        assert_eq!(tt.new_row(1, true),  &[ TreePart::Corner ]);
    }

    #[test]
    fn two_times_two_nested_children() {
        let mut tt = TreeTrunk::default();
        assert_eq!(tt.new_row(0, true), &[]);

        assert_eq!(tt.new_row(1, false), &[ TreePart::Edge ]);
        assert_eq!(tt.new_row(2, false), &[ TreePart::Line, TreePart::Edge ]);
        assert_eq!(tt.new_row(2, true),  &[ TreePart::Line, TreePart::Corner ]);

        assert_eq!(tt.new_row(1, true),  &[ TreePart::Corner ]);
        assert_eq!(tt.new_row(2, false), &[ TreePart::Blank, TreePart::Edge ]);
        assert_eq!(tt.new_row(2, true),  &[ TreePart::Blank, TreePart::Corner ]);
    }
}
