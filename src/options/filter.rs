use fs::DotFilter;
use fs::filter::{FileFilter, SortField, SortCase, IgnorePatterns};

use options::{flags, Misfire};
use options::parser::MatchedFlags;


impl FileFilter {

    /// Determines the set of file filter options to use, based on the user’s
    /// command-line arguments.
    pub fn deduce(matches: &MatchedFlags) -> Result<FileFilter, Misfire> {
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

const SORTS: &[&str] = &[ "name", "Name", "size", "extension",
                          "Extension", "modified", "accessed",
                          "created", "inode", "type", "none" ];

impl SortField {

    /// Determine the sort field to use, based on the presence of a “sort”
    /// argument. This will return `Err` if the option is there, but does not
    /// correspond to a valid field.
    fn deduce(matches: &MatchedFlags) -> Result<SortField, Misfire> {
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
    pub fn deduce(matches: &MatchedFlags) -> Result<DotFilter, Misfire> {
        match matches.count(&flags::ALL) {
            0 => Ok(DotFilter::JustFiles),
            1 => Ok(DotFilter::Dotfiles),
            _ => if matches.has(&flags::TREE) { Err(Misfire::TreeAllAll) }
                                         else { Ok(DotFilter::DotfilesAndDots) }
        }
    }
}


impl IgnorePatterns {

    /// Determines the set of file filter options to use, based on the user’s
    /// command-line arguments.
    pub fn deduce(matches: &MatchedFlags) -> Result<IgnorePatterns, Misfire> {

        let inputs = match matches.get(&flags::IGNORE_GLOB) {
            None => return Ok(IgnorePatterns::empty()),
            Some(is) => is,
        };

        let (patterns, mut errors) = IgnorePatterns::parse_from_iter(inputs.to_string_lossy().split('|'));

        // It can actually return more than one glob error,
        // but we only use one.
        if let Some(error) = errors.pop() {
            return Err(error.into())
        }
        else {
            Ok(patterns)
        }
    }
}



#[cfg(test)]
mod test {
    use super::*;
    use std::ffi::OsString;
    use options::flags;

    pub fn os(input: &'static str) -> OsString {
        let mut os = OsString::new();
        os.push(input);
        os
    }

    macro_rules! test {
        ($name:ident: $type:ident <- $inputs:expr => $result:expr) => {
            #[test]
            fn $name() {
                use options::parser::Arg;
                use options::test::assert_parses;
                use options::test::Strictnesses::*;

                static TEST_ARGS: &[&Arg] = &[ &flags::SORT, &flags::ALL, &flags::TREE, &flags::IGNORE_GLOB ];
                assert_parses($inputs.as_ref(), TEST_ARGS, Both, |mf| $type::deduce(mf), $result)
            }
        };
    }

    mod sort_fields {
        use super::*;

        // Default behaviour
        test!(empty:         SortField <- []                  => Ok(SortField::default()));

        // Sort field arguments
        test!(one_arg:       SortField <- ["--sort=cr"]       => Ok(SortField::CreatedDate));
        test!(one_long:      SortField <- ["--sort=size"]     => Ok(SortField::Size));
        test!(one_short:     SortField <- ["-saccessed"]      => Ok(SortField::AccessedDate));
        test!(lowercase:     SortField <- ["--sort", "name"]  => Ok(SortField::Name(SortCase::Sensitive)));
        test!(uppercase:     SortField <- ["--sort", "Name"]  => Ok(SortField::Name(SortCase::Insensitive)));

        // Errors
        test!(error:         SortField <- ["--sort=colour"]   => Err(Misfire::bad_argument(&flags::SORT, &os("colour"), super::SORTS)));

        // Overriding
        test!(overridden:    SortField <- ["--sort=cr",       "--sort", "mod"]     => Ok(SortField::ModifiedDate));
        test!(overridden_2:  SortField <- ["--sort", "none",  "--sort=Extension"]  => Ok(SortField::Extension(SortCase::Insensitive)));
    }


    mod dot_filters {
        use super::*;

        // Default behaviour
        test!(empty:      DotFilter <- []               => Ok(DotFilter::JustFiles));

        // --all
        test!(all:        DotFilter <- ["--all"]        => Ok(DotFilter::Dotfiles));
        test!(all_all:    DotFilter <- ["--all", "-a"]  => Ok(DotFilter::DotfilesAndDots));
        test!(all_all_2:  DotFilter <- ["-aa"]          => Ok(DotFilter::DotfilesAndDots));

        // --all and --tree
        test!(tree_a:     DotFilter <- ["-Ta"]          => Ok(DotFilter::Dotfiles));
        test!(tree_aa:    DotFilter <- ["-Taa"]         => Err(Misfire::TreeAllAll));
    }


    mod ignore_patternses {
        use super::*;
        use std::iter::FromIterator;
        use glob;

        fn pat(string: &'static str) -> glob::Pattern {
            glob::Pattern::new(string).unwrap()
        }

        // Various numbers of globs
        test!(none:   IgnorePatterns <- []                             => Ok(IgnorePatterns::empty()));
        test!(one:    IgnorePatterns <- ["--ignore-glob", "*.ogg"]     => Ok(IgnorePatterns::from_iter(vec![ pat("*.ogg") ])));
        test!(two:    IgnorePatterns <- ["--ignore-glob=*.ogg|*.MP3"]  => Ok(IgnorePatterns::from_iter(vec![ pat("*.ogg"), pat("*.MP3") ])));
        test!(loads:  IgnorePatterns <- ["-I*|?|.|*"]                  => Ok(IgnorePatterns::from_iter(vec![ pat("*"), pat("?"), pat("."), pat("*") ])));
    }
}
