use ansi_term::Style;

use output::cell::TextCell;
use fs::fields as f;


impl f::Inode {
    pub fn render(&self, style: Style) -> TextCell {
        TextCell::paint(style, self.0.to_string())
    }
}


#[cfg(test)]
pub mod test {
    use output::cell::TextCell;
    use fs::fields as f;

    use ansi_term::Colour::*;


    #[test]
    fn blocklessness() {
        let io = f::Inode(1414213);
        let expected = TextCell::paint_str(Cyan.underline(), "1414213");
        assert_eq!(expected, io.render(Cyan.underline()).into());
    }
}
