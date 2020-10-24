use crate::options::{flags, OptionsError};
use crate::options::parser::MatchedFlags;

use crate::output::file_name::{Options, Classify};


impl Options {
    pub fn deduce(matches: &MatchedFlags<'_>) -> Result<Self, OptionsError> {
        let classify = Classify::deduce(matches)?;
        let icons = matches.has(&flags::ICONS)?;

        Ok(Self { classify, icons })
    }
}

impl Classify {
    fn deduce(matches: &MatchedFlags<'_>) -> Result<Self, OptionsError> {
        let flagged = matches.has(&flags::CLASSIFY)?;

        if flagged { Ok(Self::AddFileIndicators) }
              else { Ok(Self::JustFilenames) }
    }
}
