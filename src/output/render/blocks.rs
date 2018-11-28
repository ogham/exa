use ansi_term::Style;

use fs::fields as f;
use output::cell::TextCell;

impl f::Blocks {
    pub fn render<C: Colours>(&self, colours: &C) -> TextCell {
        match *self {
            f::Blocks::Some(ref blk) => TextCell::paint(colours.block_count(), blk.to_string()),
            f::Blocks::None => TextCell::blank(colours.no_blocks()),
        }
    }
}

pub trait Colours {
    fn block_count(&self) -> Style;
    fn no_blocks(&self) -> Style;
}

#[cfg(test)]
pub mod test {
    use ansi_term::Colour::*;
    use ansi_term::Style;

    use super::Colours;
    use fs::fields as f;
    use output::cell::TextCell;

    struct TestColours;

    impl Colours for TestColours {
        fn block_count(&self) -> Style {
            Red.blink()
        }
        fn no_blocks(&self) -> Style {
            Green.italic()
        }
    }

    #[test]
    fn blocklessness() {
        let blox = f::Blocks::None;
        let expected = TextCell::blank(Green.italic());

        assert_eq!(expected, blox.render(&TestColours).into());
    }

    #[test]
    fn blockfulity() {
        let blox = f::Blocks::Some(3005);
        let expected = TextCell::paint_str(Red.blink(), "3005");

        assert_eq!(expected, blox.render(&TestColours).into());
    }
}
