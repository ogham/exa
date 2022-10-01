use ansi_term::Style;

use crate::fs::fields as f;
use crate::output::cell::TextCell;


impl f::Blocks {
    pub fn render<C: Colours>(&self, colours: &C) -> TextCell {
        match self {
            Self::Some(blk)  => TextCell::paint(colours.block_count(), blk.to_string()),
            Self::None       => TextCell::blank(colours.no_blocks()),
        }
    }
}


pub trait Colours {
    fn block_count(&self) -> Style;
    fn no_blocks(&self) -> Style;
}


#[cfg(test)]
pub mod test {
    use ansi_term::Style;
    use ansi_term::Colour::*;

    use super::Colours;
    use crate::output::cell::TextCell;
    use crate::fs::fields as f;


    struct TestColours;

    impl Colours for TestColours {
        fn block_count(&self) -> Style { Red.blink() }
        fn no_blocks(&self)   -> Style { Green.italic() }
    }


    #[test]
    fn blocklessness() {
        let blox = f::Blocks::None;
        let expected = TextCell::blank(Green.italic());

        assert_eq!(expected, blox.render(&TestColours));
    }


    #[test]
    fn blockfulity() {
        let blox = f::Blocks::Some(3005);
        let expected = TextCell::paint_str(Red.blink(), "3005");

        assert_eq!(expected, blox.render(&TestColours));
    }
}
