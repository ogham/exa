use ansi_term::{ANSIString, Style};

use crate::output::cell::{TextCell, DisplayWidth};
use crate::fs::fields as f;


impl f::Git {
    pub fn render(self, colours: &dyn Colours) -> TextCell {
        TextCell {
            width: DisplayWidth::from(2),
            contents: vec![
                self.staged.render(colours),
                self.unstaged.render(colours),
            ].into(),
        }
    }
}


impl f::GitStatus {
    fn render(self, colours: &dyn Colours) -> ANSIString<'static> {
        match self {
            Self::NotModified  => colours.not_modified().paint("-"),
            Self::New          => colours.new().paint("N"),
            Self::Modified     => colours.modified().paint("M"),
            Self::Deleted      => colours.deleted().paint("D"),
            Self::Renamed      => colours.renamed().paint("R"),
            Self::TypeChange   => colours.type_change().paint("T"),
            Self::Ignored      => colours.ignored().paint("I"),
            Self::Conflicted   => colours.conflicted().paint("U"),
        }
    }
}


pub trait Colours {
    fn not_modified(&self) -> Style;
    #[allow(clippy::new_ret_no_self)]
    fn new(&self) -> Style;
    fn modified(&self) -> Style;
    fn deleted(&self) -> Style;
    fn renamed(&self) -> Style;
    fn type_change(&self) -> Style;
    fn ignored(&self) -> Style;
    fn conflicted(&self) -> Style;
}


#[cfg(test)]
pub mod test {
    use super::Colours;
    use crate::output::cell::{TextCell, DisplayWidth};
    use crate::fs::fields as f;

    use ansi_term::Colour::*;
    use ansi_term::Style;


    struct TestColours;

    impl Colours for TestColours {
        fn not_modified(&self) -> Style { Fixed(90).normal() }
        fn new(&self)          -> Style { Fixed(91).normal() }
        fn modified(&self)     -> Style { Fixed(92).normal() }
        fn deleted(&self)      -> Style { Fixed(93).normal() }
        fn renamed(&self)      -> Style { Fixed(94).normal() }
        fn type_change(&self)  -> Style { Fixed(95).normal() }
        fn ignored(&self)      -> Style { Fixed(96).normal() }
        fn conflicted(&self)   -> Style { Fixed(97).normal() }
    }


    #[test]
    fn git_blank() {
        let stati = f::Git {
            staged:   f::GitStatus::NotModified,
            unstaged: f::GitStatus::NotModified,
        };

        let expected = TextCell {
            width: DisplayWidth::from(2),
            contents: vec![
                Fixed(90).paint("-"),
                Fixed(90).paint("-"),
            ].into(),
        };

        assert_eq!(expected, stati.render(&TestColours))
    }


    #[test]
    fn git_new_changed() {
        let stati = f::Git {
            staged:   f::GitStatus::New,
            unstaged: f::GitStatus::Modified,
        };

        let expected = TextCell {
            width: DisplayWidth::from(2),
            contents: vec![
                Fixed(91).paint("N"),
                Fixed(92).paint("M"),
            ].into(),
        };

        assert_eq!(expected, stati.render(&TestColours))
    }
}
