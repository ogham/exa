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
    pub fn ascii_art(&self) -> &'static str {
        match *self {
            TreePart::Edge    => "├──",
            TreePart::Line    => "│  ",
            TreePart::Corner  => "└──",
            TreePart::Blank   => "   ",
        }
    }
}
