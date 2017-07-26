use glob;

use fs::DotFilter;
use fs::filter::{FileFilter, SortField, SortCase, IgnorePatterns};

use options::{flags, Misfire};
use options::parser::Matches;


impl FileFilter {

    /// Determines the set of file filter options to use, based on the user’s
    /// command-line arguments.
    pub fn deduce(matches: &Matches) -> Result<FileFilter, Misfire> {
        Ok(FileFilter {
            list_dirs_first: matches.has(&flags::DIRS_FIRST),
            reverse:         matches.has(&flags::REVERSE),
            sort_field:      SortField::deduce(matches)?,
            dot_filter:      DotFilter::deduce(matches)?,
            ignore_patterns: IgnorePatterns::deduce(matches)?,
        })
    }
}



impl Default for SortField {
    fn default() -> SortField {
        SortField::Name(SortCase::Sensitive)
    }
}

impl SortField {

    /// Determine the sort field to use, based on the presence of a “sort”
    /// argument. This will return `Err` if the option is there, but does not
    /// correspond to a valid field.
    fn deduce(matches: &Matches) -> Result<SortField, Misfire> {

        const SORTS: &[&str] = &[ "name", "Name", "size", "extension",
                                  "Extension", "modified", "accessed",
                                  "created", "inode", "type", "none" ];

        let word = match matches.get(&flags::SORT) {
            Some(w)  => w,
            None     => return Ok(SortField::default()),
        };

        if word == "name" || word == "filename" {
            Ok(SortField::Name(SortCase::Sensitive))
        }
        else if word == "Name" || word == "Filename" {
            Ok(SortField::Name(SortCase::Insensitive))
        }
        else if word == "size" || word == "filesize" {
            Ok(SortField::Size)
        }
        else if word == "ext" || word == "extension" {
            Ok(SortField::Extension(SortCase::Sensitive))
        }
        else if word == "Ext" || word == "Extension" {
            Ok(SortField::Extension(SortCase::Insensitive))
        }
        else if word == "mod" || word == "modified" {
            Ok(SortField::ModifiedDate)
        }
        else if word == "acc" || word == "accessed" {
            Ok(SortField::AccessedDate)
        }
        else if word == "cr" || word == "created" {
            Ok(SortField::CreatedDate)
        }
        else if word == "inode" {
            Ok(SortField::FileInode)
        }
        else if word == "type" {
            Ok(SortField::FileType)
        }
        else if word == "none" {
            Ok(SortField::Unsorted)
        }
        else {
            Err(Misfire::bad_argument(&flags::SORT, word, SORTS))
        }
    }
}


impl DotFilter {
    pub fn deduce(matches: &Matches) -> Result<DotFilter, Misfire> {
        let dots = match matches.count(&flags::ALL) {
            0 => return Ok(DotFilter::JustFiles),
            1 => DotFilter::Dotfiles,
            _ => DotFilter::DotfilesAndDots,
        };

        if matches.has(&flags::TREE) {
            Err(Misfire::TreeAllAll)
        }
        else {
            Ok(dots)
        }
    }
}


impl IgnorePatterns {

    /// Determines the set of file filter options to use, based on the user’s
    /// command-line arguments.
    pub fn deduce(matches: &Matches) -> Result<IgnorePatterns, Misfire> {
        let patterns = match matches.get(&flags::IGNORE_GLOB) {
            None => Ok(Vec::new()),
            Some(is) => is.to_string_lossy().split('|').map(|a| glob::Pattern::new(a)).collect(),
        }?;

        // TODO: is to_string_lossy really the best way to handle
        // invalid UTF-8 there?

        Ok(IgnorePatterns { patterns })
    }
}
