use ansi_term::{ANSIString, Style, Color};

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

impl f::SubdirGitRepo {
    pub fn render(self) -> TextCell {
        let style = Style::new();
        let branch_style = match self.branch.as_deref(){
            Some("master") => style.fg(Color::Green),
            Some("main") => style.fg(Color::Green),
            Some(_) => style.fg(Color::Fixed(208)),
            _ => style,
        };
        
        let branch = branch_style.paint(self.branch.unwrap_or(String::from("-")));

        let s = match self.status {
            f::SubdirGitRepoStatus::NoRepo => style.paint("- "),
            f::SubdirGitRepoStatus::GitClean => style.bold().fg(Color::Green).paint("V "),
            f::SubdirGitRepoStatus::GitDirty => style.bold().fg(Color::Red).paint("X "),
        };

        TextCell {
            width: DisplayWidth::from(2 + branch.len()),
            contents: vec![s,branch].into(),
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

        assert_eq!(expected, stati.render(&TestColours).into())
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

        assert_eq!(expected, stati.render(&TestColours).into())
    }
}
