% exa(1) v0.9.0

<!-- This is the exa(1) man page, written in Markdown. -->
<!-- To generate the roff version, run `just man`, -->
<!-- and the man page will appear in the ‘target’ directory. -->


NAME
====

exa — a modern replacement for ls


SYNOPSIS
========

`exa [options] [files...]`

**exa** is a modern replacement for `ls`.
It uses colours for information by default, helping you distinguish between many types of files, such as whether you are the owner, or in the owning group.

It also has extra features not present in the original `ls`, such as viewing the Git status for a directory, or recursing into directories with a tree view.


EXAMPLES
========

`exa`
: Lists the contents of the current directory in a grid.

`exa --oneline --reverse --sort=size`
: Displays a list of files with the largest at the top.

`exa --long --header --inode --git`
: Displays a table of files with a header, showing each file’s metadata, inode, and Git status.

`exa --long --tree --level=3`
: Displays a tree of files, three levels deep, as well as each file’s metadata.


DISPLAY OPTIONS
===============

`-1`, `--oneline`
: Display one entry per line.

`-F`, `--classify`
: Display file kind indicators next to file names.

`-G`, `--grid`
: Display entries as a grid (default).

`-l`, `--long`
: Display extended file metadata as a table.

`-R`, `--recurse`
: Recurse into directories.

`-T`, `--tree`
: Recurse into directories as a tree.

`-x`, `--across`
: Sort the grid across, rather than downwards.

`--color`, `--colour=WHEN`
: When to use terminal colours.
Valid settings are ‘`always`’, ‘`automatic`’, and ‘`never`’.

`--color-scale`, `--colour-scale`
: Colour file sizes on a scale.

`--icons`
: Display icons next to file names.

`--no-icons`
: Don't display icons. (Always overrides --icons)


FILTERING AND SORTING OPTIONS
=============================

`-a`, `--all`
: Show hidden and “dot” files.
Use this twice to also show the ‘`.`’ and ‘`..`’ directories.

`-d`, `--list-dirs`
: List directories like regular files.

`-L`, `--level=DEPTH`
: Limit the depth of recursion.

`-r`, `--reverse`
: Reverse the sort order.

`-s`, `--sort=SORT_FIELD`
: Which field to sort by.

Valid sort fields are ‘`name`’, ‘`Name`’, ‘`extension`’, ‘`Extension`’, ‘`size`’, ‘`modified`’, ‘`changed`’, ‘`accessed`’, ‘`created`’, ‘`inode`’, ‘`type`’, and ‘`none`’.

The `modified` sort field has the aliases ‘`date`’, ‘`time`’, and ‘`newest`’, and its reverse order has the aliases ‘`age`’ and ‘`oldest`’.

Sort fields starting with a capital letter will sort uppercase before lowercase: ‘A’ then ‘B’ then ‘a’ then ‘b’. Fields starting with a lowercase letter will mix them: ‘A’ then ‘a’ then ‘B’ then ‘b’.

`-I`, `--ignore-glob=GLOBS`
: Glob patterns, pipe-separated, of files to ignore.

`--git-ignore` [if exa was built with git support]
: Do not list files that are ignored by Git.

`--group-directories-first`
: List directories before other files.

`-D`, `--only-dirs`
: List only directories, not files.


LONG VIEW OPTIONS
=================

These options are available when running with `--long` (`-l`):

`-b`, `--binary`
: List file sizes with binary prefixes.

`-B`, `--bytes`
: List file sizes in bytes, without any prefixes.

`--changed`
: Use the changed timestamp field.

`-g`, `--group`
: List each file’s group.

`-h`, `--header`
: Add a header row to each column.

`-H`, `--links`
: List each file’s number of hard links.

`-i`, `--inode`
: List each file’s inode number.

`-m`, `--modified`
: Use the modified timestamp field.

`-n`, `--numeric`
: List numeric user and group IDs.

`-S`, `--blocks`
: List each file’s number of file system blocks.

`-t`, `--time=WORD`
: Which timestamp field to list.

: Valid timestamp fields are ‘`modified`’, ‘`changed`’, ‘`accessed`’, and ‘`created`’.

`--time-style=STYLE`
: How to format timestamps.

: Valid timestamp styles are ‘`default`’, ‘`iso`’, ‘`long-iso`’, and ‘`full-iso`’.

`-u`, `--accessed`
: Use the accessed timestamp field.

`-U`, `--created`
: Use the created timestamp field.

`--no-permissions`
: Suppress the permissions field.

`--no-filesize`
: Suppress the file size field.

`--no-user`
: Suppress the user field.

`--no-time`
: Suppress the time field.

`-@`, `--extended`
: List each file’s extended attributes and sizes.

`--git`  [if exa was built with git support]
: List each file’s Git status, if tracked.


ENVIRONMENT VARIABLES
=====================

exa responds to the following environment variables:

## `COLUMNS`

Overrides the width of the terminal, in characters.

For example, ‘`COLUMNS=80 exa`’ will show a grid view with a maximum width of 80 characters.

This option won’t do anything when exa’s output doesn’t wrap, such as when using the `--long` view.

## `EXA_STRICT`

Enables _strict mode_, which will make exa error when two command-line options are incompatible.

Usually, options can override each other going right-to-left on the command line, so that exa can be given aliases: creating an alias ‘`exa=exa --sort=ext`’ then running ‘`exa --sort=size`’ with that alias will run ‘`exa --sort=ext --sort=size`’, and the sorting specified by the user will override the sorting specified by the alias.

In strict mode, the two options will not co-operate, and exa will error.

This option is intended for use with automated scripts and other situations where you want to be certain you’re typing in the right command.

## `EXA_GRID_ROWS`

Limits the grid-details view (‘`exa --grid --long`’) so it’s only activated when at least the given number of rows of output would be generated.

With widescreen displays, it’s possible for the grid to look very wide and sparse, on just one or two lines with none of the columns lining up.
By specifying a minimum number of rows, you can only use the view if it’s going to be worth using.

## `EXA_ICON_SPACING`

Specifies the number of spaces to print between an icon (see the ‘`--icons`’ option) and its file name.

Different terminals display icons differently, as they usually take up more than one character width on screen, so there’s no “standard” number of spaces that exa can use to separate an icon from text. One space may place the icon too close to the text, and two spaces may place it too far away. So the choice is left up to the user to configure depending on their terminal emulator.

## `LS_COLORS`, `EXA_COLORS`

Specifies the colour scheme used to highlight files based on their name and kind, as well as highlighting metadata and parts of the UI.

For more information on the format of these environment variables, see the `exa_colors(5)` manual page.


EXIT STATUSES
=============

0
: If everything goes OK.

1
: If there was an I/O error during operation.

3
: If there was a problem with the command-line arguments.


AUTHOR
======

exa is maintained by Benjamin ‘ogham’ Sago and many other contributors.

**Website:** `https://the.exa.website/` \
**Source code:** `https://github.com/ogham/exa` \
**Contributors:** `https://github.com/ogham/exa/graphs/contributors`


SEE ALSO
========

- `exa_colors(5)`
