use output::cell::TextCell;
use output::colours::Colours;
use fs::fields as f;


impl f::Blocks {
    pub fn render(&self, colours: &Colours) -> TextCell {
        match *self {
            f::Blocks::Some(ref blk)  => TextCell::paint(colours.blocks, blk.to_string()),
            f::Blocks::None           => TextCell::blank(colours.punctuation),
        }
    }
}


#[cfg(test)]
pub mod test {
    use output::details::Details;
    use output::cell::TextCell;
    use fs::fields as f;

    use ansi_term::Colour::*;


    #[test]
    fn blocklessness() {
        let mut details = Details::default();
        details.colours.punctuation = Green.italic();

        let blox = f::Blocks::None;
        let expected = TextCell::blank(Green.italic());
        assert_eq!(expected, blox.render(&details.colours).into());
    }

    #[test]
    fn blockfulity() {
        let mut details = Details::default();
        details.colours.blocks = Red.blink();

        let blox = f::Blocks::Some(3005);
        let expected = TextCell::paint_str(Red.blink(), "3005");
        assert_eq!(expected, blox.render(&details.colours).into());
    }
}
