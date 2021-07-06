use crate::options::{flags, OptionsError, NumberSource};
use crate::options::parser::MatchedFlags;
use crate::options::vars::{self, Vars};

use crate::output::file_name::{Options, Classify, UseIcons};


impl Options {
    pub fn deduce<V: Vars>(matches: &MatchedFlags<'_>, vars: &V, is_a_tty: bool) -> Result<Self, OptionsError> {
        let classify = Classify::deduce(matches)?;
        let show_icons = UseIcons::deduce(matches, vars)?;

        Ok(Self { classify, show_icons, is_a_tty })
    }
}

impl Classify {
    fn deduce(matches: &MatchedFlags<'_>) -> Result<Self, OptionsError> {
        let flagged = matches.has(&flags::CLASSIFY)?;

        if flagged { Ok(Self::AddFileIndicators) }
              else { Ok(Self::JustFilenames) }
    }
}

impl UseIcons {
    pub fn deduce<V: Vars>(matches: &MatchedFlags<'_>, vars: &V) -> Result<Self, OptionsError> {
        enum AlwaysOrAuto { Always, Automatic }

        let mode_opt = matches.get(&flags::ICONS)?;
        if matches.has(&flags::NO_ICONS)? || (!matches.has(&flags::ICONS)? && mode_opt.is_none()) {
            return Ok(Self::Never);
        }

        let mode = match mode_opt {
            Some(word) => match word.to_str() {
                Some("always") => AlwaysOrAuto::Always,
                Some("auto") | Some("automatic") => AlwaysOrAuto::Automatic,
                Some("never") => return Ok(Self::Never),
                _ => return Err(OptionsError::BadArgument(&flags::COLOR, word.into()))
            }
            None => AlwaysOrAuto::Automatic,
        };

        let width = if let Some(columns) = vars.get(vars::EXA_ICON_SPACING).and_then(|s| s.into_string().ok()) {
            match columns.parse() {
                Ok(width) => width,
                Err(e) => {
                    let source = NumberSource::Env(vars::EXA_ICON_SPACING);
                    return Err(OptionsError::FailedParse(columns, source, e));
                }
            }
        } else {
            1
        };

        match mode {
            AlwaysOrAuto::Always => Ok(Self::Always(width)),
            AlwaysOrAuto::Automatic => Ok(Self::Automatic(width)),
        }
    }
}
