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
//! ```text
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
//! ```
//!
//! Creating the table like this means that each file has to be tested to see
//! if it’s the last one in the group. This is usually done by putting all the
//! files in a vector beforehand, getting its length, then comparing the index
//! of each file to see if it’s the last one. (As some files may not be
//! successfully `stat`ted, we don’t know how many files are going to exist in
//! each directory)


#[derive(PartialEq, Eq, Debug, Copy, Clone)]
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
    pub fn ascii_art(self) -> &'static str {
        match self {
            Self::Edge    => "├──",
            Self::Line    => "│  ",
            Self::Corner  => "└──",
            Self::Blank   => "   ",
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
    last_params: Option<TreeParams>,
}

#[derive(Debug, Copy, Clone)]
pub struct TreeParams {

    /// How many directories deep into the tree structure this is. Directories
    /// on top have depth 0.
    depth: TreeDepth,

    /// Whether this is the last entry in the directory.
    last: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct TreeDepth(pub usize);

impl TreeTrunk {

    /// Calculates the tree parts for an entry at the given depth and
    /// last-ness. The depth is used to determine where in the stack the tree
    /// part should be inserted, and the last-ness is used to determine which
    /// type of tree part to insert.
    ///
    /// This takes a `&mut self` because the results of each file are stored
    /// and used in future rows.
    pub fn new_row(&mut self, params: TreeParams) -> &[TreePart] {

        // If this isn’t our first iteration, then update the tree parts thus
        // far to account for there being another row after it.
        if let Some(last) = self.last_params {
            self.stack[last.depth.0] = if last.last { TreePart::Blank }
                                               else { TreePart::Line };
        }

        // Make sure the stack has enough space, then add or modify another
        // part into it.
        self.stack.resize(params.depth.0 + 1, TreePart::Edge);
        self.stack[params.depth.0] = if params.last { TreePart::Corner }
                                               else { TreePart::Edge };

        self.last_params = Some(params);

        // Return the tree parts as a slice of the stack.
        //
        // Ignore the first element here to prevent a ‘zeroth level’ from
        // appearing before the very first directory. This level would
        // join unrelated directories without connecting to anything:
        //
        //     with [0..]        with [1..]
        //     ==========        ==========
        //      ├── folder        folder
        //      │  └── file       └── file
        //      └── folder        folder
        //         └── file       └──file
        //
        &self.stack[1..]
    }
}

impl TreeParams {
    pub fn new(depth: TreeDepth, last: bool) -> Self {
        Self { depth, last }
    }

    pub fn is_at_root(&self) -> bool {
        self.depth.0 == 0
    }
}

impl TreeDepth {
    pub fn root() -> Self {
        Self(0)
    }

    pub fn deeper(self) -> Self {
        Self(self.0 + 1)
    }

    /// Creates an iterator that, as well as yielding each value, yields a
    /// `TreeParams` with the current depth and last flag filled in.
    pub fn iterate_over<I, T>(self, inner: I) -> Iter<I>
    where I: ExactSizeIterator + Iterator<Item = T>
    {
        Iter { current_depth: self, inner }
    }
}


pub struct Iter<I> {
    current_depth: TreeDepth,
    inner: I,
}

impl<I, T> Iterator for Iter<I>
where I: ExactSizeIterator + Iterator<Item = T>
{
    type Item = (TreeParams, T);

    fn next(&mut self) -> Option<Self::Item> {
        let t = self.inner.next()?;

        // TODO: use exact_size_is_empty API soon
        let params = TreeParams::new(self.current_depth, self.inner.len() == 0);
        Some((params, t))
    }
}


#[cfg(test)]
mod trunk_test {
    use super::*;

    fn params(depth: usize, last: bool) -> TreeParams {
        TreeParams::new(TreeDepth(depth), last)
    }

    #[test]
    fn empty_at_first() {
        let mut tt = TreeTrunk::default();
        assert_eq!(tt.new_row(params(0, true)),  &[ ]);
    }

    #[test]
    fn one_child() {
        let mut tt = TreeTrunk::default();
        assert_eq!(tt.new_row(params(0, true)),  &[ ]);
        assert_eq!(tt.new_row(params(1, true)),  &[ TreePart::Corner ]);
    }

    #[test]
    fn two_children() {
        let mut tt = TreeTrunk::default();
        assert_eq!(tt.new_row(params(0, true)),  &[ ]);
        assert_eq!(tt.new_row(params(1, false)), &[ TreePart::Edge ]);
        assert_eq!(tt.new_row(params(1, true)),  &[ TreePart::Corner ]);
    }

    #[test]
    fn two_times_two_children() {
        let mut tt = TreeTrunk::default();
        assert_eq!(tt.new_row(params(0, false)), &[ ]);
        assert_eq!(tt.new_row(params(1, false)), &[ TreePart::Edge ]);
        assert_eq!(tt.new_row(params(1, true)),  &[ TreePart::Corner ]);

        assert_eq!(tt.new_row(params(0, true)),  &[ ]);
        assert_eq!(tt.new_row(params(1, false)), &[ TreePart::Edge ]);
        assert_eq!(tt.new_row(params(1, true)),  &[ TreePart::Corner ]);
    }

    #[test]
    fn two_times_two_nested_children() {
        let mut tt = TreeTrunk::default();
        assert_eq!(tt.new_row(params(0, true)),  &[ ]);

        assert_eq!(tt.new_row(params(1, false)), &[ TreePart::Edge ]);
        assert_eq!(tt.new_row(params(2, false)), &[ TreePart::Line, TreePart::Edge ]);
        assert_eq!(tt.new_row(params(2, true)),  &[ TreePart::Line, TreePart::Corner ]);

        assert_eq!(tt.new_row(params(1, true)),  &[ TreePart::Corner ]);
        assert_eq!(tt.new_row(params(2, false)), &[ TreePart::Blank, TreePart::Edge ]);
        assert_eq!(tt.new_row(params(2, true)),  &[ TreePart::Blank, TreePart::Corner ]);
    }
}


#[cfg(test)]
mod iter_test {
    use super::*;

    #[test]
    fn test_iteration() {
        let foos = &[ "first", "middle", "last" ];
        let mut iter = TreeDepth::root().iterate_over(foos.iter());

        let next = iter.next().unwrap();
        assert_eq!(&"first", next.1);
        assert!(!next.0.last);

        let next = iter.next().unwrap();
        assert_eq!(&"middle", next.1);
        assert!(!next.0.last);

        let next = iter.next().unwrap();
        assert_eq!(&"last", next.1);
        assert!(next.0.last);

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_empty() {
        let nothing: &[usize] = &[];
        let mut iter = TreeDepth::root().iterate_over(nothing.iter());
        assert!(iter.next().is_none());
    }
}
