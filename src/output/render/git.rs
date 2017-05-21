use output::cell::{TextCell, DisplayWidth};
use output::colours::Colours;
use fs::fields as f;


impl f::Git {
    pub fn render(&self, colours: &Colours) -> TextCell {
        let git_char = |status| match status {
            &f::GitStatus::NotModified  => colours.punctuation.paint("-"),
            &f::GitStatus::New          => colours.git.new.paint("N"),
            &f::GitStatus::Modified     => colours.git.modified.paint("M"),
            &f::GitStatus::Deleted      => colours.git.deleted.paint("D"),
            &f::GitStatus::Renamed      => colours.git.renamed.paint("R"),
            &f::GitStatus::TypeChange   => colours.git.typechange.paint("T"),
        };

        TextCell {
            width: DisplayWidth::from(2),
            contents: vec![
                git_char(&self.staged),
                git_char(&self.unstaged)
            ].into(),
        }
    }
}


#[cfg(test)]
pub mod test {
    use output::details::Details;
    use output::cell::{TextCell, DisplayWidth};
    use fs::fields as f;

    use ansi_term::Colour::*;


    #[test]
    fn git_blank() {
        let mut details = Details::default();
        details.colours.punctuation = Fixed(44).normal();

        let stati = f::Git {
            staged:   f::GitStatus::NotModified,
            unstaged: f::GitStatus::NotModified,
        };

        let expected = TextCell {
            width: DisplayWidth::from(2),
            contents: vec![
                Fixed(44).paint("-"),
                Fixed(44).paint("-"),
            ].into(),
        };

        assert_eq!(expected, stati.render(&details.colours).into())
    }


    #[test]
    fn git_new_changed() {
        let mut details = Details::default();
        details.colours.git.new = Red.normal();
        details.colours.git.modified = Purple.normal();

        let stati = f::Git {
            staged:   f::GitStatus::New,
            unstaged: f::GitStatus::Modified,
        };

        let expected = TextCell {
            width: DisplayWidth::from(2),
            contents: vec![
                Red.paint("N"),
                Purple.paint("M"),
            ].into(),
        };

        assert_eq!(expected, stati.render(&details.colours).into())
    }
}
