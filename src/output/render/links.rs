use output::cell::TextCell;
use output::colours::Colours;
use fs::fields as f;

use locale;


impl f::Links {
    pub fn render(&self, colours: &Colours, numeric: &locale::Numeric) -> TextCell {
        let style = if self.multiple { colours.links.multi_link_file }
                                else { colours.links.normal };

        TextCell::paint(style, numeric.format_int(self.count))
    }
}


#[cfg(test)]
pub mod test {
    use output::colours::Colours;
    use output::cell::{TextCell, DisplayWidth};
    use fs::fields as f;

    use ansi_term::Colour::*;
    use locale;


    #[test]
    fn regular_file() {
        let mut colours = Colours::default();
        colours.links.normal = Blue.normal();

        let stati = f::Links {
            count:    1,
            multiple: false,
        };

        let expected = TextCell {
            width: DisplayWidth::from(1),
            contents: vec![ Blue.paint("1") ].into(),
        };

        assert_eq!(expected, stati.render(&colours, &locale::Numeric::english()).into());
    }

    #[test]
    fn regular_directory() {
        let mut colours = Colours::default();
        colours.links.normal = Blue.normal();

        let stati = f::Links {
            count:    3005,
            multiple: false,
        };

        let expected = TextCell {
            width: DisplayWidth::from(5),
            contents: vec![ Blue.paint("3,005") ].into(),
        };

        assert_eq!(expected, stati.render(&colours, &locale::Numeric::english()).into());
    }

    #[test]
    fn popular_file() {
        let mut colours = Colours::default();
        colours.links.multi_link_file = Blue.on(Red);

        let stati = f::Links {
            count:    3005,
            multiple: true,
        };

        let expected = TextCell {
            width: DisplayWidth::from(5),
            contents: vec![ Blue.on(Red).paint("3,005") ].into(),
        };

        assert_eq!(expected, stati.render(&colours, &locale::Numeric::english()).into());
    }
}
