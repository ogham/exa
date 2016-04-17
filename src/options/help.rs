
pub static OPTIONS: &'static str = r##"
DISPLAY OPTIONS
  -1, --oneline      display one entry per line
  -G, --grid         display entries in a grid view (default)
  -l, --long         display extended details and attributes
  -R, --recurse      recurse into directories
  -T, --tree         recurse into subdirectories in a tree view
  -x, --across       sort multi-column view entries across
  --color, --colour  when to colourise the output

FILTERING AND SORTING OPTIONS
  -a, --all                  show dot-files
  -d, --list-dirs            list directories as regular files
  -r, --reverse              reverse order of files
  -s, --sort WORD            field to sort by
  --group-directories-first  list directories before other files
"##;

pub static LONG_OPTIONS: &'static str = r##"
LONG VIEW OPTIONS
  -b, --binary       use binary prefixes in file sizes
  -B, --bytes        list file sizes in bytes, without prefixes
  -g, --group        show group as well as user
  -h, --header       show a header row at the top
  -H, --links        show number of hard links
  -i, --inode        show each file's inode number
  -L, --level DEPTH  maximum depth of recursion
  -m, --modified     display timestamp of most recent modification
  -S, --blocks       show number of file system blocks
  -t, --time WORD    which timestamp to show for a file
  -u, --accessed     display timestamp of last access for a file
  -U, --created      display timestamp of creation for a file
"##;

pub static GIT_HELP:      &'static str = r##"  --git              show git status for files"##;
pub static EXTENDED_HELP: &'static str = r##"  -@, --extended     display extended attribute keys and sizes"##;