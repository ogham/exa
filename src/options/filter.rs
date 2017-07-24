use getopts;
use glob;

use fs::DotFilter;
use fs::filter::{FileFilter, SortField, SortCase, IgnorePatterns};
use options::misfire::Misfire;


impl FileFilter {

    /// Determines the set of file filter options to use, based on the user’s
    /// command-line arguments.
    pub fn deduce(matches: &getopts::Matches) -> Result<FileFilter, Misfire> {
        Ok(FileFilter {
            list_dirs_first: matches.opt_present("group-directories-first"),
            reverse:         matches.opt_present("reverse"),
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
    fn deduce(matches: &getopts::Matches) -> Result<SortField, Misfire> {

        const SORTS: &[&str] = &[ "name", "Name", "size", "extension",
                                  "Extension", "modified", "accessed",
                                  "created", "inode", "type", "none" ];

        if let Some(word) = matches.opt_str("sort") {
            match &*word {
                "name" | "filename"   => Ok(SortField::Name(SortCase::Sensitive)),
                "Name" | "Filename"   => Ok(SortField::Name(SortCase::Insensitive)),
                "size" | "filesize"   => Ok(SortField::Size),
                "ext"  | "extension"  => Ok(SortField::Extension(SortCase::Sensitive)),
                "Ext"  | "Extension"  => Ok(SortField::Extension(SortCase::Insensitive)),
                "mod"  | "modified"   => Ok(SortField::ModifiedDate),
                "acc"  | "accessed"   => Ok(SortField::AccessedDate),
                "cr"   | "created"    => Ok(SortField::CreatedDate),
                "inode"               => Ok(SortField::FileInode),
                "type"                => Ok(SortField::FileType),
                "none"                => Ok(SortField::Unsorted),
                field                 => Err(Misfire::bad_argument("sort", field, SORTS))
            }
        }
        else {
            Ok(SortField::default())
        }
    }
}


impl DotFilter {
    pub fn deduce(matches: &getopts::Matches) -> Result<DotFilter, Misfire> {
        let dots = match matches.opt_count("all") {
            0 => return Ok(DotFilter::JustFiles),
            1 => DotFilter::Dotfiles,
            _ => DotFilter::DotfilesAndDots,
        };

        if matches.opt_present("tree") {
            Err(Misfire::Useless("all --all", true, "tree"))
        }
        else {
            Ok(dots)
        }
    }
}


impl IgnorePatterns {

    /// Determines the set of file filter options to use, based on the user’s
    /// command-line arguments.
    pub fn deduce(matches: &getopts::Matches) -> Result<IgnorePatterns, Misfire> {
        let patterns = match matches.opt_str("ignore-glob") {
            None => Ok(Vec::new()),
            Some(is) => is.split('|').map(|a| glob::Pattern::new(a)).collect(),
        };

        Ok(IgnorePatterns {
            patterns: patterns?,
        })
    }
}
