#!/bin/bash
set +xe


# Release mode
case "$1" in
  "--release") exa_binary="$HOME/target/release/exa" ;;
  *)           exa_binary="$HOME/target/debug/exa" ;;
esac

if [ ! -f "$exa_binary" ]; then
  echo "exa binary ($exa_binary) does not exist"
  if [ "$1" != "--release" ]; then echo -e "create it first with \033[1;32mbuild-exa\033[0m or \033[1;32mb\033[0m"; fi
  exit 1
fi


# The exa command that ends up being run
exa="$exa_binary --colour=always"

# Directory containing our awkward testcase files
testcases="/testcases"

# Directory containing existing test results to compare against
results="/vagrant/xtests"


# We want to use strict mode here. It’s important that no combination of
# testing flags happens to work by accident!
export EXA_STRICT="1"

# We also don’t want to see reams and reams of debug output.
export EXA_DEBUG=""

# And default colours by default
export LS_COLORS=""
export EXA_COLORS=""


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


# File size tests
$exa $testcases/files -l --binary | diff -q - $results/files_l_binary  || exit 1
$exa $testcases/files -l --bytes  | diff -q - $results/files_l_bytes   || exit 1

EXA_STRICT= $exa $testcases/files -l --bytes --binary   | diff -q - $results/files_l_binary  || exit 1
EXA_STRICT= $exa $testcases/files -l --binary --bytes   | diff -q - $results/files_l_bytes   || exit 1


# Attributes
# (there are many tests, but they’re all done in one go)
$exa $testcases/attributes  -l@T | diff -q - $results/attributes        || exit 1
$exa $testcases/attributes   -@T | diff -q - $results/attributes_dir    || exit 1
$exa $testcases/attributes/* -@T | diff -q - $results/attributes_files  || exit 1


# UIDs and GIDs
$exa $testcases/passwd -lgh | diff -q - $results/passwd  || exit 1


# Permissions, and current users and groups
sudo -u cassowary $exa $testcases/permissions -lghR 2>&1 | diff -q - $results/permissions_sudo  || exit 1
                  $exa $testcases/permissions -lghR 2>&1 | diff -q - $results/permissions       || exit 1

# File names
# (Mostly escaping control characters in file names)
COLUMNS=80 $exa $testcases/file-names     2>&1 | diff -q - $results/file_names    || exit 1
COLUMNS=80 $exa $testcases/file-names -x  2>&1 | diff -q - $results/file_names_x  || exit 1
COLUMNS=80 $exa $testcases/file-names -R  2>&1 | diff -q - $results/file_names_R  || exit 1
           $exa $testcases/file-names -1  2>&1 | diff -q - $results/file_names_1  || exit 1
           $exa $testcases/file-names -T  2>&1 | diff -q - $results/file_names_T  || exit 1
           $exa $testcases/file-names -T@ 2>&1 | diff -q - $results/file_names_T@ || exit 1

# At least make sure it handles invalid UTF-8 arguments without crashing
$exa $testcases/file-names/* >/dev/null || exit 1


# Sorting and extension file types
$exa $testcases/file-names-exts -1 2>&1 --sort=name | diff -q - $results/file-names-exts           || exit 1
$exa $testcases/file-names-exts -1 2>&1 --sort=Name | diff -q - $results/file-names-exts-case      || exit 1
$exa $testcases/file-names-exts -1 2>&1 --sort=ext  | diff -q - $results/file-names-exts-ext       || exit 1
$exa $testcases/file-names-exts -1 2>&1 --sort=Ext  | diff -q - $results/file-names-exts-ext-case  || exit 1

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
$exa $testcases/dates -lh            --sort=newest   2>&1 | diff -q - $results/dates_modified  || exit 1
$exa $testcases/dates -lh         -r --sort=newest   2>&1 | diff -q - $results/dates_deifidom  || exit 1
$exa $testcases/dates -lh            --sort=oldest   2>&1 | diff -q - $results/dates_deifidom  || exit 1
$exa $testcases/dates -l       --time-style=long-iso 2>&1 | diff -q - $results/dates_long_iso  || exit 1
$exa $testcases/dates -l       --time-style=full-iso 2>&1 | diff -q - $results/dates_full_iso  || exit 1
$exa $testcases/dates -l       --time-style=iso      2>&1 | diff -q - $results/dates_iso       || exit 1

# Locales
# These two are used in particular because they have 5-long and 4-long
# month names respectively
env LC_ALL=fr_FR.UTF-8 LANG=fr_FR.UTF-8 $exa $testcases/dates -l | diff -q - $results/dates_fr  || exit 1
env LC_ALL=ja_JP.UTF-8 LANG=ja_JP.UTF-8 $exa $testcases/dates -l | diff -q - $results/dates_jp  || exit 1


# Paths and directories
# These directories are created in the VM user’s home directory (the default
# location) when a Cargo build is done.
(cd; mkdir -p target/debug/build
     $exa -1d target target/debug target/debug/build | diff -q - $results/dir_paths)  || exit 1
     $exa -1d . .. /                                 | diff -q - $results/dirs        || exit 1


# Links
COLUMNS=80 $exa $testcases/links     2>&1 | diff -q - $results/links          || exit 1
           $exa $testcases/links -1  2>&1 | diff -q - $results/links_1        || exit 1
           $exa $testcases/links -T  2>&1 | diff -q - $results/links_T        || exit 1
           $exa $testcases/links -T@ 2>&1 | diff -q - $results/links_T@       || exit 1
           $exa /proc/1/root     -T  2>&1 | diff -q - $results/proc_1_root    || exit 1
           $exa /proc/1/root     -T@ 2>&1 | diff -q - $results/proc_1_root_@  || exit 1

# There’ve been bugs where the target file wasn’t printed properly when the
# symlink file was specified on the command-line directly.
$exa $testcases/links/* -1d | diff -q - $results/links_1_files || exit 1


# Colours and terminals
# Just because COLUMNS is present, doesn’t mean output is to a terminal
COLUMNS=80 $exa_binary                    $testcases/files -l | diff -q - $results/files_l_bw  || exit 1
COLUMNS=80 $exa_binary --colour=always    $testcases/files -l | diff -q - $results/files_l     || exit 1
COLUMNS=80 $exa_binary --colour=never     $testcases/files -l | diff -q - $results/files_l_bw  || exit 1
COLUMNS=80 $exa_binary --colour=automatic $testcases/files -l | diff -q - $results/files_l_bw  || exit 1

# Switching colour off
COLUMNS=80 $exa_binary --colour=never    $testcases/file-names       | diff -q - $results/file_names_bw       || exit 1
COLUMNS=80 $exa_binary --colour=never    $testcases/file-names-exts  | diff -q - $results/file-names-exts-bw  || exit 1
COLUMNS=80 $exa_binary --colour=never -T $testcases/file-names/links | diff -q - $results/links_bw            || exit 1


# Git
$exa $testcases/git                      -l --git 2>&1 | diff -q - $results/git_1_long       || exit 1
$exa $testcases/git                      -l       2>&1 | diff -q - $results/git_1_nogit      || exit 1
$exa $testcases/git            --recurse -l --git 2>&1 | diff -q - $results/git_1_recurse    || exit 1
$exa $testcases/git               --tree -l --git 2>&1 | diff -q - $results/git_1_tree       || exit 1
$exa $testcases/git/moves/thither --tree -l --git 2>&1 | diff -q - $results/git_1_file       || exit 1
$exa $testcases/git/additions            -l --git 2>&1 | diff -q - $results/git_1_additions  || exit 1
$exa $testcases/git/edits                -l --git 2>&1 | diff -q - $results/git_1_edits      || exit 1
$exa $testcases/git/{additions,edits}    -l --git 2>&1 | diff -q - $results/git_1_both       || exit 1

$exa $testcases/git2                          -l --git 2>&1 | diff -q - $results/git_2_long        || exit 1
$exa $testcases/git2                          -l       2>&1 | diff -q - $results/git_2_nogit       || exit 1
$exa $testcases/git2                   --tree -l --git 2>&1 | diff -q - $results/git_2_tree        || exit 1
$exa $testcases/git2                --recurse -l --git 2>&1 | diff -q - $results/git_2_recurse     || exit 1
$exa $testcases/git2/ignoreds                 -l --git 2>&1 | diff -q - $results/git_2_ignoreds    || exit 1
$exa $testcases/git2/target                   -l --git 2>&1 | diff -q - $results/git_2_target      || exit 1
$exa $testcases/git2/deeply/nested/repository -l --git 2>&1 | diff -q - $results/git_2_repository  || exit 1
$exa $testcases/git2/{deeply,ignoreds,target} -l --git 2>&1 | diff -q - $results/git_2_all         || exit 1

# Regressions test
$exa $testcases/git3 -l --git &>/dev/null || exit 1

COLUMNS=150 $exa $testcases/git/**/* $testcases --git --long --grid -d | diff -q - $results/git_1_files  || exit 1

$exa $testcases/git $testcases/git2 --git --long | diff -q - $results/git_12  || exit 1

$exa $testcases/git/additions $testcases/git2/deeply \
     $testcases/git/edits     $testcases/git2/deeply/nested  --git --long | diff -q - $results/git_1212  || exit 1

$exa $testcases/git2/deeply/nested/directory $testcases/git/edits \
     $testcases/git2/target $testcases/git2/deeply $testcases/git  --git --long | diff -q - $results/git_21221  || exit 1

            $exa $testcases/files -l  --git | diff -q - $results/files_l      || exit 1    # no git status for dirs
COLUMNS=40  $exa $testcases/files -lG --git | diff -q - $results/files_lG_40  || exit 1    # that aren't under git


# .gitignore
$exa $testcases/git2 --recurse --long --git-ignore 2>&1 | diff - $results/git_2_ignore_recurse
$exa $testcases/git2    --tree --long --git-ignore 2>&1 | diff - $results/git_2_ignore_tree


# Hidden files
COLUMNS=80 $exa $testcases/hiddens     2>&1 | diff -q - $results/hiddens     || exit 1
COLUMNS=80 $exa $testcases/hiddens -a  2>&1 | diff -q - $results/hiddens_a   || exit 1
COLUMNS=80 $exa $testcases/hiddens -aa 2>&1 | diff -q - $results/hiddens_aa  || exit 1

$exa $testcases/hiddens -l     2>&1 | diff -q - $results/hiddens_l    || exit 1
$exa $testcases/hiddens -l -a  2>&1 | diff -q - $results/hiddens_la   || exit 1
$exa $testcases/hiddens -l -aa 2>&1 | diff -q - $results/hiddens_laa  || exit 1


# Themes
LS_COLORS="bd=31:cd=32:pi=34"  $exa -1 $testcases/specials 2>&1 | diff -q - $results/themed_specials  || exit 1
EXA_COLORS="bd=31:cd=32:pi=34" $exa -1 $testcases/specials 2>&1 | diff -q - $results/themed_specials  || exit 1

             LS_COLORS="*.deb=1;37:*.tar.*=1;37" $exa -1 $testcases/file-names-exts/compressed.* 2>&1 | diff -q - $results/themed_compresseds  || exit 1
EXA_COLORS="*.deb=1;37:*.tar.*=1;37"             $exa -1 $testcases/file-names-exts/compressed.* 2>&1 | diff -q - $results/themed_compresseds  || exit 1
EXA_COLORS="*.deb=1;37" LS_COLORS="*.tar.*=1;37" $exa -1 $testcases/file-names-exts/compressed.* 2>&1 | diff -q - $results/themed_compresseds  || exit 1

 LS_COLORS="reset:*.deb=1;37:*.tar.*=1;37"       $exa -1 $testcases/file-names-exts/compressed.* 2>&1 | diff -q - $results/themed_compresseds    || exit 1
EXA_COLORS="reset:*.deb=1;37:*.tar.*=1;37"       $exa -1 $testcases/file-names-exts/compressed.* 2>&1 | diff -q - $results/themed_compresseds_r  || exit 1

EXA_COLORS="or=32:bO=1:cc=35:ln=31:xx=33"        $exa -1 $testcases/file-names/links 2>&1     | diff -q - $results/themed_links  || exit 1

# EXA_COLORS overrides LS_COLORS
LS_COLORS="bd=32:cd=34:pi=31" EXA_COLORS="bd=31:cd=32:pi=34" $exa -1 $testcases/specials 2>&1 | diff -q - $results/themed_specials  || exit 1

EXA_COLORS="di=38;5;195:fi=38;5;250:xx=38;5;237:ur=38;5;194:uw=38;5;193:ux=38;5;192:gr=38;5;191:gw=38;5;190:gx=38;5;118:tr=38;5;119:tw=38;5;120:tx=38;5;121:su=38;5;51:sf=38;5;50:sn=38;5;49:un=38;5;46:da=38;5;47:ex=38;5;48" \
    $exa --long $testcases/permissions 2>&1 | diff -q - $results/themed_long  || exit 1

EXA_COLORS="reset" $exa $testcases/file-names-exts -1  2>&1 | diff -q - $results/themed_un  || exit 1

# Errors
$exa --binary           2>&1 | diff -q - $results/error_useless     || exit 1
$exa --ternary          2>&1 | diff -q - $results/error_long        || exit 1
$exa -4                 2>&1 | diff -q - $results/error_short       || exit 1
$exa --time             2>&1 | diff -q - $results/error_value       || exit 1
$exa --long=time        2>&1 | diff -q - $results/error_overvalued  || exit 1
$exa -l --long          2>&1 | diff -q - $results/error_duplicate   || exit 1
$exa -ll                2>&1 | diff -q - $results/error_twice       || exit 1
$exa -l --time-style=24 2>&1 | diff -q - $results/error_setting     || exit 1

# Error suggestions
$exa -ltr 2>&1 | diff -q - $results/error_ltr  || exit 1
$exa -lt  2>&1 | diff -q - $results/error_lt   || exit 1


# Debug mode
# What gets logged keeps changing, so instead of checking for an exact output,
# list an empty directory and fail if nothing gets printed.
DEBUG_OUT=$(EXA_DEBUG="1" $exa $testcases/attributes/dirs/no-xattrs_empty -lh 2>&1)
if [ -z "$DEBUG_OUT" ]; then
    echo "Debug test produced no output"
    exit 1
fi


# And finally...
$exa --help        | diff -q - $results/help       || exit 1
$exa --help --long | diff -q - $results/help_long  || exit 1


echo "All the tests passed!"
