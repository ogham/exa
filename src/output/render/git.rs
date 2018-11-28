use ansi_term::{ANSIString, Style};

use fs::fields as f;
use output::cell::{DisplayWidth, TextCell};

impl f::Git {
    pub fn render(&self, colours: &Colours) -> TextCell {
        TextCell {
            width: DisplayWidth::from(2),
            contents: vec![self.staged.render(colours), self.unstaged.render(colours)].into(),
        }
    }
}

impl f::GitStatus {
    fn render(&self, colours: &Colours) -> ANSIString<'static> {
        match *self {
            f::GitStatus::NotModified => colours.not_modified().paint("-"),
            f::GitStatus::New => colours.new().paint("N"),
            f::GitStatus::Modified => colours.modified().paint("M"),
            f::GitStatus::Deleted => colours.deleted().paint("D"),
            f::GitStatus::Renamed => colours.renamed().paint("R"),
            f::GitStatus::TypeChange => colours.type_change().paint("T"),
        }
    }
}

pub trait Colours {
    fn not_modified(&self) -> Style;
    fn new(&self) -> Style;
    fn modified(&self) -> Style;
    fn deleted(&self) -> Style;
    fn renamed(&self) -> Style;
    fn type_change(&self) -> Style;
}

#[cfg(test)]
pub mod test {
    use super::Colours;
    use fs::fields as f;
    use output::cell::{DisplayWidth, TextCell};

    use ansi_term::Colour::*;
    use ansi_term::Style;

    struct TestColours;

    impl Colours for TestColours {
        fn not_modified(&self) -> Style {
            Fixed(90).normal()
        }
        fn new(&self) -> Style {
            Fixed(91).normal()
        }
        fn modified(&self) -> Style {
            Fixed(92).normal()
        }
        fn deleted(&self) -> Style {
            Fixed(93).normal()
        }
        fn renamed(&self) -> Style {
            Fixed(94).normal()
        }
        fn type_change(&self) -> Style {
            Fixed(95).normal()
        }
    }

    #[test]
    fn git_blank() {
        let stati = f::Git {
            staged: f::GitStatus::NotModified,
            unstaged: f::GitStatus::NotModified,
        };

        let expected = TextCell {
            width: DisplayWidth::from(2),
            contents: vec![Fixed(90).paint("-"), Fixed(90).paint("-")].into(),
        };

        assert_eq!(expected, stati.render(&TestColours).into())
    }

    #[test]
    fn git_new_changed() {
        let stati = f::Git {
            staged: f::GitStatus::New,
            unstaged: f::GitStatus::Modified,
        };

        let expected = TextCell {
            width: DisplayWidth::from(2),
            contents: vec![Fixed(91).paint("N"), Fixed(92).paint("M")].into(),
        };

        assert_eq!(expected, stati.render(&TestColours).into())
    }
}
