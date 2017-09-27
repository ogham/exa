#compdef exa

__exa() {
    _arguments \
        "(- 1 *)"{-v,--version}"[Show version of exa]" \
        "(- 1 *)"{-\?,--help}"[Show list of command-line options]" \
        {-1,--oneline}"[Display one entry per line]" \
        {-l,--long}"[Display extended file metadata as a table]" \
        {-G,--grid}"[Display entries as a grid]" \
        {-x,--across}"[Sort the grid across, rather than downwards]" \
        {-R,--recurse}"[Recurse into directories]" \
        {-T,--tree}"[Recurse into directories as a tree]" \
        {-F,--classify}"[Display type indicator by file names]" \
        {--color,--colour}"[When to use terminal colours]" \
        {--color,--colour}-scale"[Highlight levels of file sizes distinctly]" \
        --group-directories-first"[Sort directories before other files]" \
        --git-ignore"[Ignore files mentioned in '.gitignore']" \
        {-a,--all}"[Show hidden and 'dot' files]" \
        {-d,--list-dirs}"[List directories like regular files]" \
        {-L,--level}"+[Limit the depth of recursion]" \
        {-r,--reverse}"[Reverse the sort order]" \
        {-s,--sort}"[Which field to sort by]:(sort field):(accessed age created date extension Extension filename Filename inode modified oldest name Name newest none size time type)" \
        {-I,--ignore-glob}"[Ignore files that match these glob patterns]" \
        {-b,--binary}"[List file sizes with binary prefixes]" \
        {-B,--bytes}"[List file sizes in bytes, without any prefixes]" \
        {-g,--group}"[List each file's group]" \
        {-h,--header}"[Add a header row to each column]" \
        {-H,--links}"[List each file's number of hard links]" \
        {-i,--inode}"[List each file's inode number]" \
        {-m,--modified}"[Use the modified timestamp field]" \
        {-S,--blocks}"[List each file's number of filesystem blocks]" \
        {-t,--time}"[Which time field to show]:(time field):(accessed created modified)" \
        --time-style"[How to format timestamps]:(time style):(default iso long-iso full-iso)" \
        {-u,--accessed}"[Use the accessed timestamp field]" \
        {-U,--created}"[Use the created timestamp field]" \
        --git"[List each file's Git status, if tracked]" \
        {-@,--extended}"[List each file's extended attributes and sizes]" \
        '*:filename:_files'
}

__exa
