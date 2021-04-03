#compdef exa

# Save this file as _exa in /usr/local/share/zsh/site-functions or in any
# other folder in $fpath.  E. g. save it in a folder called ~/.zfunc and add a
# line containing `fpath=(~/.zfunc $fpath)` somewhere before `compinit` in your
# ~/.zshrc.

__exa() {
    # Give completions using the `_arguments` utility function with
    # `-s` for option stacking like `exa -ab` for `exa -a -b` and
    # `-S` for delimiting options with `--` like in `exa -- -a`.
    _arguments -s -S \
        "(- *)"{-v,--version}"[Show version of exa]" \
        "(- *)"{-'\?',--help}"[Show list of command-line options]" \
        {-1,--oneline}"[Display one entry per line]" \
        {-l,--long}"[Display extended file metadata as a table]" \
        {-G,--grid}"[Display entries as a grid]" \
        {-x,--across}"[Sort the grid across, rather than downwards]" \
        {-R,--recurse}"[Recurse into directories]" \
        {-T,--tree}"[Recurse into directories as a tree]" \
        {-F,--classify}"[Display type indicator by file names]" \
        --colo{,u}r"[When to use terminal colours]" \
        --colo{,u}r-scale"[Highlight levels of file sizes distinctly]" \
        --icons"[Display icons]" \
        --no-icons"[Hide icons]" \
        --group-directories-first"[Sort directories before other files]" \
        --git-ignore"[Ignore files mentioned in '.gitignore']" \
        {-a,--all}"[Show hidden and 'dot' files]" \
        {-d,--list-dirs}"[List directories like regular files]" \
        {-D,--only-dirs}"[List only directories]" \
        {-L,--level}"+[Limit the depth of recursion]" \
        {-r,--reverse}"[Reverse the sort order]" \
        {-s,--sort}="[Which field to sort by]:(sort field):(accessed age changed created date extension Extension filename Filename inode modified oldest name Name newest none size time type)" \
        {-I,--ignore-glob}"[Ignore files that match these glob patterns]" \
        {-b,--binary}"[List file sizes with binary prefixes]" \
        {-B,--bytes}"[List file sizes in bytes, without any prefixes]" \
        --changed"[Use the changed timestamp field]" \
        {-g,--group}"[List each file's group]" \
        {-h,--header}"[Add a header row to each column]" \
        {-H,--links}"[List each file's number of hard links]" \
        {-i,--inode}"[List each file's inode number]" \
        {-m,--modified}"[Use the modified timestamp field]" \
        {-n,--numeric}"[List numeric user and group IDs.]" \
        {-S,--blocks}"[List each file's number of filesystem blocks]" \
        {-t,--time}="[Which time field to show]:(time field):(accessed changed created modified)" \
        --time-style="[How to format timestamps]:(time style):(default iso long-iso full-iso)" \
        --no-permissions"[Suppress the permissions field]" \
        --octal-permissions"[List each file's permission in octal format]" \
        --no-filesize"[Suppress the filesize field]" \
        --no-user"[Suppress the user field]" \
        --no-time"[Suppress the time field]" \
        {-u,--accessed}"[Use the accessed timestamp field]" \
        {-U,--created}"[Use the created timestamp field]" \
        --git"[List each file's Git status, if tracked]" \
        {-@,--extended}"[List each file's extended attributes and sizes]" \
        '*:filename:_files'
}

__exa
