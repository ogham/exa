use std::cmp::Ordering;
use std::os::unix::fs::MetadataExt;

use getopts;
use glob;
use natord;

use fs::File;
use options::misfire::Misfire;


/// The **file filter** processes a vector of files before outputting them,
/// filtering and sorting the files depending on the user’s command-line
/// flags.
#[derive(Default, PartialEq, Debug, Clone)]
pub struct FileFilter {

    /// Whether directories should be listed first, and other types of file
    /// second. Some users prefer it like this.
    pub list_dirs_first: bool,

    /// The metadata field to sort by.
    pub sort_field: SortField,

    /// Whether to reverse the sorting order. This would sort the largest
    /// files first, or files starting with Z, or the most-recently-changed
    /// ones, depending on the sort field.
    pub reverse: bool,

    /// Whether to include invisible “dot” files when listing a directory.
    ///
    /// Files starting with a single “.” are used to determine “system” or
    /// “configuration” files that should not be displayed in a regular
    /// directory listing.
    ///
    /// This came about more or less by a complete historical accident,
    /// when the original `ls` tried to hide `.` and `..`:
    /// https://plus.google.com/+RobPikeTheHuman/posts/R58WgWwN9jp
    ///
    ///   When one typed ls, however, these files appeared, so either Ken or
    ///   Dennis added a simple test to the program. It was in assembler then,
    ///   but the code in question was equivalent to something like this:
    ///      if (name[0] == '.') continue;
    ///   This statement was a little shorter than what it should have been,
    ///   which is:
    ///      if (strcmp(name, ".") == 0 || strcmp(name, "..") == 0) continue;
    ///   but hey, it was easy.
    ///
    ///   Two things resulted.
    ///
    ///   First, a bad precedent was set. A lot of other lazy programmers
    ///   introduced bugs by making the same simplification. Actual files
    ///   beginning with periods are often skipped when they should be counted.
    ///
    ///   Second, and much worse, the idea of a "hidden" or "dot" file was
    ///   created. As a consequence, more lazy programmers started dropping
    ///   files into everyone's home directory. I don't have all that much
    ///   stuff installed on the machine I'm using to type this, but my home
    ///   directory has about a hundred dot files and I don't even know what
    ///   most of them are or whether they're still needed. Every file name
    ///   evaluation that goes through my home directory is slowed down by
    ///   this accumulated sludge.
    show_invisibles: bool,

    /// Glob patterns to ignore. Any file name that matches *any* of these
    /// patterns won't be displayed in the list.
    ignore_patterns: IgnorePatterns,
}

impl FileFilter {

    /// Determines the set of file filter options to use, based on the user’s
    /// command-line arguments.
    pub fn deduce(matches: &getopts::Matches) -> Result<FileFilter, Misfire> {
        Ok(FileFilter {
            list_dirs_first: matches.opt_present("group-directories-first"),
            reverse:         matches.opt_present("reverse"),
            sort_field:      try!(SortField::deduce(matches)),
            show_invisibles: matches.opt_present("all"),
            ignore_patterns: try!(IgnorePatterns::deduce(matches)),
        })
    }

    /// Remove every file in the given vector that does *not* pass the
    /// filter predicate for files found inside a directory.
    pub fn filter_child_files(&self, files: &mut Vec<File>) {
        if !self.show_invisibles {
            files.retain(|f| !f.is_dotfile());
        }

        files.retain(|f| !self.ignore_patterns.is_ignored(f));
    }

    /// Remove every file in the given vector that does *not* pass the
    /// filter predicate for file names specified on the command-line.
    ///
    /// The rules are different for these types of files than the other
    /// type because the ignore rules can be used with globbing. For
    /// example, running "exa -I='*.tmp' .vimrc" shouldn't filter out the
    /// dotfile, because it's been directly specified. But running
    /// "exa -I='*.ogg' music/*" should filter out the ogg files obtained
    /// from the glob, even though the globbing is done by the shell!
    pub fn filter_argument_files(&self, files: &mut Vec<File>) {
        files.retain(|f| !self.ignore_patterns.is_ignored(f));
    }

    /// Sort the files in the given vector based on the sort field option.
    pub fn sort_files<'a, F>(&self, files: &mut Vec<F>)
    where F: AsRef<File<'a>> {

        files.sort_by(|a, b| self.compare_files(a.as_ref(), b.as_ref()));

        if self.reverse {
            files.reverse();
        }

        if self.list_dirs_first {
            // This relies on the fact that `sort_by` is stable.
            files.sort_by(|a, b| b.as_ref().is_directory().cmp(&a.as_ref().is_directory()));
        }
    }

    /// Compares two files to determine the order they should be listed in,
    /// depending on the search field.
    pub fn compare_files(&self, a: &File, b: &File) -> Ordering {
        use self::SortCase::{Sensitive, Insensitive};

        match self.sort_field {
            SortField::Unsorted  => Ordering::Equal,

            SortField::Name(Sensitive)    => natord::compare(&a.name, &b.name),
            SortField::Name(Insensitive)  => natord::compare_ignore_case(&a.name, &b.name),

            SortField::Size          => a.metadata.len().cmp(&b.metadata.len()),
            SortField::FileInode     => a.metadata.ino().cmp(&b.metadata.ino()),
            SortField::ModifiedDate  => a.metadata.mtime().cmp(&b.metadata.mtime()),
            SortField::AccessedDate  => a.metadata.atime().cmp(&b.metadata.atime()),
            SortField::CreatedDate   => a.metadata.ctime().cmp(&b.metadata.ctime()),

            SortField::Extension(Sensitive) => match a.ext.cmp(&b.ext) {
                Ordering::Equal  => natord::compare(&*a.name, &*b.name),
                order            => order,
            },

            SortField::Extension(Insensitive) => match a.ext.cmp(&b.ext) {
                Ordering::Equal  => natord::compare_ignore_case(&*a.name, &*b.name),
                order            => order,
            },
        }
    }
}


/// User-supplied field to sort by.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum SortField {

    /// Don't apply any sorting. This is usually used as an optimisation in
    /// scripts, where the order doesn't matter.
    Unsorted,

    /// The file name. This is the default sorting.
    Name(SortCase),

    /// The file's extension, with extensionless files being listed first.
    Extension(SortCase),

    /// The file's size.
    Size,

    /// The file's inode. This is sometimes analogous to the order in which
    /// the files were created on the hard drive.
    FileInode,

    /// The time at which this file was modified (the `mtime`).
    ///
    /// As this is stored as a Unix timestamp, rather than a local time
    /// instance, the time zone does not matter and will only be used to
    /// display the timestamps, not compare them.
    ModifiedDate,

    /// The time at this file was accessed (the `atime`).
    ///
    /// Oddly enough, this field rarely holds the *actual* accessed time.
    /// Recording a read time means writing to the file each time it’s read
    /// slows the whole operation down, so many systems will only update the
    /// timestamp in certain circumstances. This has become common enough that
    /// it’s now expected behaviour for the `atime` field.
    /// http://unix.stackexchange.com/a/8842
    AccessedDate,

    /// The time at which this file was changed or created (the `ctime`).
    ///
    /// Contrary to the name, this field is used to mark the time when a
    /// file's metadata changed -- its permissions, owners, or link count.
    ///
    /// In original Unix, this was, however, meant as creation time.
    /// https://www.bell-labs.com/usr/dmr/www/cacm.html
    CreatedDate,
}

/// Whether a field should be sorted case-sensitively or case-insensitively.
///
/// This determines which of the `natord` functions to use.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum SortCase {

    /// Sort files case-sensitively with uppercase first, with ‘A’ coming
    /// before ‘a’.
    Sensitive,

    /// Sort files case-insensitively, with ‘A’ being equal to ‘a’.
    Insensitive,
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
                "none"                => Ok(SortField::Unsorted),
                "inode"               => Ok(SortField::FileInode),
                field                 => Err(Misfire::bad_argument("sort", field, &[
                                            "name", "Name", "size", "extension", "Extension",
                                            "modified", "accessed", "created", "inode", "none"]
                ))
            }
        }
        else {
            Ok(SortField::default())
        }
    }
}


#[derive(PartialEq, Default, Debug, Clone)]
struct IgnorePatterns {
    patterns: Vec<glob::Pattern>,
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
            patterns: try!(patterns),
        })
    }

    fn is_ignored(&self, file: &File) -> bool {
        self.patterns.iter().any(|p| p.matches(&file.name))
    }
}
