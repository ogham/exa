use output::cell::TextCell;
use output::colours::Colours;
use fs::fields as f;


impl f::Inode {
    pub fn render(&self, colours: &Colours) -> TextCell {
        TextCell::paint(colours.inode, self.0.to_string())
    }
}


#[cfg(test)]
pub mod test {
    use output::colours::Colours;
    use output::cell::TextCell;
    use fs::fields as f;

    use ansi_term::Colour::*;


    #[test]
    fn blocklessness() {
        let mut colours = Colours::default();
        colours.inode = Cyan.underline();

        let io = f::Inode(1414213);
        let expected = TextCell::paint_str(Cyan.underline(), "1414213");
        assert_eq!(expected, io.render(&colours).into());
    }
}
