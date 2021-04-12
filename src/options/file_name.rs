use crate::options::{flags, OptionsError, NumberSource};
use crate::options::parser::MatchedFlags;
use crate::options::vars::{self, Vars};

use crate::output::file_name::{Options, Classify, ShowIcons};


impl Options {
    pub fn deduce<V: Vars>(matches: &MatchedFlags<'_>, vars: &V) -> Result<Self, OptionsError> {
        let classify = Classify::deduce(matches)?;
        let show_icons = ShowIcons::deduce(matches, vars)?;

        Ok(Self { classify, show_icons })
    }
}

impl Classify {
    fn deduce(matches: &MatchedFlags<'_>) -> Result<Self, OptionsError> {
        let flagged = matches.has(&flags::CLASSIFY)?;

        if flagged { Ok(Self::AddFileIndicators) }
              else { Ok(Self::JustFilenames) }
    }
}

impl ShowIcons {
    pub fn deduce<V: Vars>(matches: &MatchedFlags<'_>, vars: &V) -> Result<Self, OptionsError> {
        if matches.has(&flags::NO_ICONS)? || !matches.has(&flags::ICONS)? {
            Ok(Self::Off)
        }
        else if let Some(columns) = vars.get(vars::EXA_ICON_SPACING).and_then(|s| s.into_string().ok()) {
            match columns.parse() {
                Ok(width) => {
                    Ok(Self::On(width))
                }
                Err(e) => {
                    let source = NumberSource::Env(vars::EXA_ICON_SPACING);
                    Err(OptionsError::FailedParse(columns, source, e))
                }
            }
        }
        else {
            Ok(Self::On(1))
        }
    }
}
