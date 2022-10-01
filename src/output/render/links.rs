use ansi_term::Style;
use locale::Numeric as NumericLocale;

use crate::fs::fields as f;
use crate::output::cell::TextCell;


impl f::Links {
    pub fn render<C: Colours>(&self, colours: &C, numeric: &NumericLocale) -> TextCell {
        let style = if self.multiple { colours.multi_link_file() }
                                else { colours.normal() };

        TextCell::paint(style, numeric.format_int(self.count))
    }
}


pub trait Colours {
    fn normal(&self) -> Style;
    fn multi_link_file(&self) -> Style;
}


#[cfg(test)]
pub mod test {
    use super::Colours;
    use crate::output::cell::{TextCell, DisplayWidth};
    use crate::fs::fields as f;

    use ansi_term::Colour::*;
    use ansi_term::Style;
    use locale;


    struct TestColours;

    impl Colours for TestColours {
        fn normal(&self)           -> Style { Blue.normal() }
        fn multi_link_file(&self)  -> Style { Blue.on(Red) }
    }


    #[test]
    fn regular_file() {
        let stati = f::Links {
            count:    1,
            multiple: false,
        };

        let expected = TextCell {
            width: DisplayWidth::from(1),
            contents: vec![ Blue.paint("1") ].into(),
        };

        assert_eq!(expected, stati.render(&TestColours, &locale::Numeric::english()));
    }

    #[test]
    fn regular_directory() {
        let stati = f::Links {
            count:    3005,
            multiple: false,
        };

        let expected = TextCell {
            width: DisplayWidth::from(5),
            contents: vec![ Blue.paint("3,005") ].into(),
        };

        assert_eq!(expected, stati.render(&TestColours, &locale::Numeric::english()));
    }

    #[test]
    fn popular_file() {
        let stati = f::Links {
            count:    3005,
            multiple: true,
        };

        let expected = TextCell {
            width: DisplayWidth::from(5),
            contents: vec![ Blue.on(Red).paint("3,005") ].into(),
        };

        assert_eq!(expected, stati.render(&TestColours, &locale::Numeric::english()));
    }
}
