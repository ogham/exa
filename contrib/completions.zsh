#compdef exa

__exa() {
    _arguments \
        "(- 1 *)"{-v,--version}"[Show version of exa]" \
        "(- 1 *)"{-\?,--help}"[Show list of command-line options]" \
        {-1,--oneline}"[Display one entry per line]" \
        {-G,--grid}"[Display entries as a grid]" \
        {-l,--long}"[Display extended metadata as a table]" \
        {-R,--recurse}"[Recurse into directories]" \
        {-T,--tree}"[Recurse into directories as a tree]" \
        {-F,--classify}"[Display type indicator by file names]" \
        {--color,--colour}"[When to use terminal colours]" \
        {--color,--colour}-scale"[Highlight levels of file sizes distinctly]" \
        --group-directories-first"[Sort directories before other files]" \
        {-a,--all}"[Don't hide hidden and 'dot' files]" \
        {-d,--list-dirs}"[List directories like regular files]" \
        {-r,--reverse}"[Reverse the sort order]" \
        {-s,--sort}"[Which field to sort by]:(sort field):(accessed created extension Extension filename Filename inode modified name Name none size)" \
        {-I,--ignore-glob}"[Ignore files that match these glob patterns]" \
        {-b,--binary}"[Display file sizes with binary prefixes]" \
        {-B,--bytes}"[Display file sizes in bytes, without any prefixes]" \
        {-g,--group}"[Show each file's group]" \
        {-h,--header}"[Add a header row to each column]" \
        {-H,--links}"[Show each file's number of hard links]" \
        {-i,--inode}"[Show each file's inode number]" \
        {-L,--level}"+[Limit the depth of recursion]" \
        {-m,--modified}"[Use the modified timestamp field]" \
        {-S,--blocks}"[Show each file's number of filesystem blocks]" \
        {-t,--time}"[Which time field to show]:(time field):(accessed created modified)" \
        {-u,--accessed}"[Use the accessed timestamp field]" \
        {-U,--created}"[Use the created timestamp field]" \
        {-U,--created}"[Show each file's Git status, if tracked]" \
        {-@,--extended}"[Show each file's extended attributes and sizes]" \
        '*:filename:_files'
}

__exa
