# Meta-stuff
complete -c exa -s 'v' -l 'version' -d "Show version of exa"
complete -c exa -s '?' -l 'help'    -d "Show list of command-line options"

# Display options
complete -c exa -s '1' -l 'oneline'      -d "Display one entry per line"
complete -c exa -s 'l' -l 'long'         -d "Display extended file metadata as a table"
complete -c exa -s 'G' -l 'grid'         -d "Display entries in a grid"
complete -c exa -s 'x' -l 'across'       -d "Sort the grid across, rather than downwards"
complete -c exa -s 'R' -l 'recurse'      -d "Recurse into directories"
complete -c exa -s 'T' -l 'tree'         -d "Recurse into directories as a tree"
complete -c exa -s 'F' -l 'classify'     -d "Display type indicator by file names"
complete -c exa        -l 'color'        -d "When to use terminal colours"
complete -c exa        -l 'colour'       -d "When to use terminal colours"
complete -c exa        -l 'color-scale'  -d "Highlight levels of file sizes distinctly"
complete -c exa        -l 'colour-scale' -d "Highlight levels of file sizes distinctly"
complete -c exa        -l 'icons'        -d "Display icons"
complete -c exa        -l 'no-icons'     -d "Don't display icons"

# Filtering and sorting options
complete -c exa -l 'group-directories-first' -d "Sort directories before other files"
complete -c exa -l 'git-ignore'           -d "Ignore files mentioned in '.gitignore'"
complete -c exa -s 'a' -l 'all'       -d "Show hidden and 'dot' files"
complete -c exa -s 'd' -l 'list-dirs' -d "List directories like regular files"
complete -c exa -s 'L' -l 'level'     -d "Limit the depth of recursion" -a "1 2 3 4 5 6 7 8 9"
complete -c exa -s 'r' -l 'reverse'   -d "Reverse the sort order"
complete -c exa -s 's' -l 'sort'   -x -d "Which field to sort by" -a "
    accessed\t'Sort by file accessed time'
    age\t'Sort by file modified time (newest first)'
    changed\t'Sort by changed time'
    created\t'Sort by file modified time'
    date\t'Sort by file modified time'
    ext\t'Sort by file extension'
    Ext\t'Sort by file extension (uppercase first)'
    extension\t'Sort by file extension'
    Extension\t'Sort by file extension (uppercase first)'
    filename\t'Sort by filename'
    Filename\t'Sort by filename (uppercase first)'
    inode\t'Sort by file inode'
    modified\t'Sort by file modified time'
    name\t'Sort by filename'
    Name\t'Sort by filename (uppercase first)'
    newest\t'Sort by file modified time (newest first)'
    none\t'Do not sort files at all'
    oldest\t'Sort by file modified time'
    size\t'Sort by file size'
    time\t'Sort by file modified time'
    type\t'Sort by file type'
"

complete -c exa -s 'I' -l 'ignore-glob' -d "Ignore files that match these glob patterns" -r
complete -c exa -s 'D' -l 'only-dirs'   -d "List only directories"

# Long view options
complete -c exa -s 'b' -l 'binary'   -d "List file sizes with binary prefixes"
complete -c exa -s 'B' -l 'bytes'    -d "List file sizes in bytes, without any prefixes"
complete -c exa -s 'g' -l 'group'    -d "List each file's group"
complete -c exa -s 'h' -l 'header'   -d "Add a header row to each column"
complete -c exa -s 'h' -l 'links'    -d "List each file's number of hard links"
complete -c exa -s 'g' -l 'group'    -d "List each file's inode number"
complete -c exa -s 'S' -l 'blocks'   -d "List each file's number of filesystem blocks"
complete -c exa -s 't' -l 'time'  -x -d "Which timestamp field to list" -a "
    modified\t'Display modified time'
    changed\t'Display changed time'
    accessed\t'Display accessed time'
    created\t'Display created time'
"
complete -c exa -s 'm' -l 'modified'      -d "Use the modified timestamp field"
complete -c exa -s 'n' -l 'numeric'       -d "List numeric user and group IDs."
complete -c exa        -l 'changed'       -d "Use the changed timestamp field"
complete -c exa -s 'u' -l 'accessed'      -d "Use the accessed timestamp field"
complete -c exa -s 'U' -l 'created'       -d "Use the created timestamp field"
complete -c exa        -l 'time-style' -x -d "How to format timestamps" -a "
    default\t'Use the default time style'
    iso\t'Display brief ISO timestamps'
    long-iso\t'Display longer ISO timestaps, up to the minute'
    full-iso\t'Display full ISO timestamps, up to the nanosecond'
"
complete -c exa        -l 'no-permissions' -d "Suppress the permissions field"
complete -c exa        -l 'octal-permissions' -d "List each file's permission in octal format"
complete -c exa        -l 'no-filesize'    -d "Suppress the filesize field"
complete -c exa        -l 'no-user'        -d "Suppress the user field"
complete -c exa        -l 'no-time'        -d "Suppress the time field"

# Optional extras
complete -c exa -l 'git' -d "List each file's Git status, if tracked"
complete -c exa -s '@' -l 'extended' -d "List each file's extended attributes and sizes"
