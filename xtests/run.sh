#!/bin/bash
set +xe


# The exa binary
exa_binary="$HOME/target/debug/exa"

# The exa command that ends up being run
exa="$exa_binary --colour=always"

# Directory containing our awkward testcase files
testcases="/testcases"

# Directory containing existing test results to compare against
results="/vagrant/xtests"


# Check that no files were created more than a year ago.
# Files not from the current year use a different date format, meaning
# that tests will fail until the VM gets re-provisioned.
# (Ignore the folder that deliberately has dates in the past)
sudo find $testcases -mtime +365 -not -path "*/dates/*" -printf "File %p has not been modified since %TY! Consider re-provisioning; tests will probably fail.\n"


# Long view tests
$exa $testcases/files -l   | diff -q - $results/files_l     || exit 1
$exa $testcases/files -lh  | diff -q - $results/files_lh    || exit 1
$exa $testcases/files -lhb | diff -q - $results/files_lhb   || exit 1
$exa $testcases/files -lhB | diff -q - $results/files_lhb2  || exit 1
$exa $testcases/attributes/dirs/no-xattrs_empty -lh | diff -q - $results/empty  || exit 1

$exa --color-scale         $testcases/files -l | diff -q - $results/files_l_scale  || exit 1


# Grid view tests
COLUMNS=40  $exa $testcases/files | diff -q - $results/files_40   || exit 1
COLUMNS=80  $exa $testcases/files | diff -q - $results/files_80   || exit 1
COLUMNS=120 $exa $testcases/files | diff -q - $results/files_120  || exit 1
COLUMNS=160 $exa $testcases/files | diff -q - $results/files_160  || exit 1
COLUMNS=200 $exa $testcases/files | diff -q - $results/files_200  || exit 1

COLUMNS=100 $exa $testcases/files/* | diff -q - $results/files_star_100  || exit 1
COLUMNS=150 $exa $testcases/files/* | diff -q - $results/files_star_150  || exit 1
COLUMNS=200 $exa $testcases/files/* | diff -q - $results/files_star_200  || exit 1


# Long grid view tests
COLUMNS=40  $exa $testcases/files -lG | diff -q - $results/files_lG_40   || exit 1
COLUMNS=80  $exa $testcases/files -lG | diff -q - $results/files_lG_80   || exit 1
COLUMNS=120 $exa $testcases/files -lG | diff -q - $results/files_lG_120  || exit 1
COLUMNS=160 $exa $testcases/files -lG | diff -q - $results/files_lG_160  || exit 1
COLUMNS=200 $exa $testcases/files -lG | diff -q - $results/files_lG_200  || exit 1

COLUMNS=100 $exa $testcases/files/* -lG | diff -q - $results/files_star_lG_100  || exit 1
COLUMNS=150 $exa $testcases/files/* -lG | diff -q - $results/files_star_lG_150  || exit 1
COLUMNS=200 $exa $testcases/files/* -lG | diff -q - $results/files_star_lG_200  || exit 1


# Attributes
# (there are many tests, but they're all done in one go)
$exa $testcases/attributes -l@T | diff -q - $results/attributes  || exit 1


# UIDs and GIDs
$exa $testcases/passwd -lgh | diff -q - $results/passwd  || exit 1


# Permissions, and current users and groups
sudo -u cassowary $exa $testcases/permissions -lghR 2>&1 | diff -q - $results/permissions_sudo  || exit 1
                  $exa $testcases/permissions -lghR 2>&1 | diff -q - $results/permissions       || exit 1

# File names
# (Mostly escaping control characters in file names)
COLUMNS=80 $exa $testcases/file-names    2>&1 | diff -q - $results/file_names   || exit 1
COLUMNS=80 $exa $testcases/file-names -x 2>&1 | diff -q - $results/file_names_x || exit 1
COLUMNS=80 $exa $testcases/file-names -R 2>&1 | diff -q - $results/file_names_R || exit 1
           $exa $testcases/file-names -1 2>&1 | diff -q - $results/file_names_1 || exit 1
           $exa $testcases/file-names -T 2>&1 | diff -q - $results/file_names_T || exit 1

# At least make sure it handles invalid UTF-8 arguments without crashing
$exa $testcases/file-names/* 2>/dev/null


# Sorting and extension file types
$exa $testcases/file-names-exts -1 2>&1 --sort=Name | diff -q - $results/file-names-exts           || exit 1
$exa $testcases/file-names-exts -1 2>&1 --sort=name | diff -q - $results/file-names-exts-case      || exit 1
$exa $testcases/file-names-exts -1 2>&1 --sort=Ext  | diff -q - $results/file-names-exts-ext       || exit 1
$exa $testcases/file-names-exts -1 2>&1 --sort=ext  | diff -q - $results/file-names-exts-ext-case  || exit 1

# Pass multiple input arguments because there aren’t enough of different types
# in one directory already
$exa $testcases/links -1 --sort=type 2>&1 | diff -q - $results/sort-by-type  || exit 1

# We can’t guarantee inode numbers, but we can at least check that they’re in
# order. The inode column is the leftmost one, so sort works for this.
$exa $testcases/file-names-exts --long --inode --sort=inode | sort --check  || exit 1


# Other file types
$exa $testcases/specials             -l 2>&1 | diff -q - $results/specials       || exit 1
$exa $testcases/specials          -F -l 2>&1 | diff -q - $results/specials_F     || exit 1
$exa $testcases/specials --sort=type -1 2>&1 | diff -q - $results/specials_sort  || exit 1


# Ignores
$exa $testcases/file-names-exts/music.* -I "*.OGG"       -1 2>&1 | diff -q - $results/ignores_ogg  || exit 1
$exa $testcases/file-names-exts/music.* -I "*.OGG|*.mp3" -1 2>&1 | diff -q - $results/empty        || exit 1


# Dates and times
$exa $testcases/dates -lh --accessed --sort=accessed 2>&1 | diff -q - $results/dates_accessed  || exit 1
$exa $testcases/dates -lh            --sort=modified 2>&1 | diff -q - $results/dates_modified  || exit 1
$exa $testcases/dates -l       --time-style=long-iso 2>&1 | diff -q - $results/dates_long_iso  || exit 1
$exa $testcases/dates -l       --time-style=full-iso 2>&1 | diff -q - $results/dates_full_iso  || exit 1
$exa $testcases/dates -l            --time-style=iso 2>&1 | diff -q - $results/dates_iso       || exit 1


# Paths and directories
# These directories are created in the VM user’s home directory (the default
# location) when a Cargo build is done.
(cd; $exa -1d target target/debug target/debug/build | diff -q - $results/dir_paths) || exit 1
     $exa -1d . .. /                                 | diff -q - $results/dirs       || exit 1


# Links
COLUMNS=80 $exa $testcases/links    2>&1 | diff -q - $results/links        || exit 1
           $exa $testcases/links -1 2>&1 | diff -q - $results/links_1      || exit 1
           $exa $testcases/links -T 2>&1 | diff -q - $results/links_T      || exit 1
           $exa /proc/1/root     -T 2>&1 | diff -q - $results/proc_1_root  || exit 1

# There’ve been bugs where the target file wasn’t printed properly when the
# symlink file was specified on the command-line directly.
$exa $testcases/links/* -1 | diff -q - $results/links_1_files || exit 1


# Colours and terminals
# Just because COLUMNS is present, doesn’t mean output is to a terminal
COLUMNS=80 $exa_binary                    $testcases/files -l | diff -q - $results/files_l_bw  || exit 1
COLUMNS=80 $exa_binary --colour=always    $testcases/files -l | diff -q - $results/files_l     || exit 1
COLUMNS=80 $exa_binary --colour=never     $testcases/files -l | diff -q - $results/files_l_bw  || exit 1
COLUMNS=80 $exa_binary --colour=automatic $testcases/files -l | diff -q - $results/files_l_bw  || exit 1


# Git
$exa $testcases/git/additions -l --git 2>&1 | diff -q - $results/git_additions  || exit 1
$exa $testcases/git/edits     -l --git 2>&1 | diff -q - $results/git_edits      || exit 1


# Hidden files
COLUMNS=80 $exa $testcases/hiddens     2>&1 | diff -q - $results/hiddens     || exit 1
COLUMNS=80 $exa $testcases/hiddens -a  2>&1 | diff -q - $results/hiddens_a   || exit 1
COLUMNS=80 $exa $testcases/hiddens -aa 2>&1 | diff -q - $results/hiddens_aa  || exit 1

$exa $testcases/hiddens -l     2>&1 | diff -q - $results/hiddens_l    || exit 1
$exa $testcases/hiddens -l -a  2>&1 | diff -q - $results/hiddens_la   || exit 1
$exa $testcases/hiddens -l -aa 2>&1 | diff -q - $results/hiddens_laa  || exit 1


# And finally...
$exa --help        | diff -q - $results/help      || exit 1
$exa --help --long | diff -q - $results/help_long || exit 1


echo "All the tests passed!"
