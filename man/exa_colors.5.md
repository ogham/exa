% exa_colors(5) v0.9.0

<!-- This is the exa_colors(5) man page, written in Markdown. -->
<!-- To generate the roff version, run `just man`, -->
<!-- and the man page will appear in the ‘target’ directory. -->


NAME
====

exa_colors — customising the file and UI colours of exa


SYNOPSIS
========

The `EXA_COLORS` environment variable can be used to customise the colours that `exa` uses to highlight file names, file metadata, and parts of the UI.

You can use the `dircolors` program to generate a script that sets the variable from an input file, or if you don’t mind editing long strings of text, you can just type it out directly. These variables have the following structure:

- A list of key-value pairs separated by ‘`=`’, such as ‘`*.txt=32`’.
- Multiple ANSI formatting codes are separated by ‘`;`’, such as ‘`*.txt=32;1;4`’.
- Finally, multiple pairs are separated by ‘`:`’, such as ‘`*.txt=32:*.mp3=1;35`’.

The key half of the pair can either be a two-letter code or a file glob, and anything that’s not a valid code will be treated as a glob, including keys that happen to be two letters long.


EXAMPLES
========

`EXA_COLORS="uu=0:gu=0"`
: Disable the “current user” highlighting

`EXA_COLORS="da=32"`
: Turn the date column green

`EXA_COLORS="Vagrantfile=1;4;33"`
: Highlight Vagrantfiles

`EXA_COLORS="*.zip=38;5;125"`
: Override the existing zip colour

`EXA_COLORS="*.md=38;5;121:*.log=38;5;248"`
: Markdown files a shade of green, log files a shade of grey


LIST OF CODES
=============

`LS_COLORS` can use these ten codes:

`di`
: directories

`ex`
: executable files

`fi`
: regular files

`pi`
: named pipes

`so`
: sockets

`bd`
: block devices

`cd`
: character devices

`ln`
: symlinks

`or`
: symlinks with no target


`EXA_COLORS` can use many more:

`ur`
: the user-read permission bit

`uw`
: the user-write permission bit

`ux`
: the user-execute permission bit for regular files

`ue`
: the user-execute for other file kinds

`gr`
: the group-read permission bit

`gw`
: the group-write permission bit

`gx`
: the group-execute permission bit

`tr`
: the others-read permission bit

`tw`
: the others-write permission bit

`tx`
: the others-execute permission bit

`su`
: setuid, setgid, and sticky permission bits for files

`sf`
: setuid, setgid, and sticky for other file kinds

`xa`
: the extended attribute indicator

`sn`
: the numbers of a file’s size (sets `nb`, `nk`, `nm`, `ng` and `nh`)

`nb`
: the numbers of a file’s size if it is lower than 1 KB/Kib

`nk`
: the numbers of a file’s size if it is between 1 KB/KiB and 1 MB/MiB

`nm`
: the numbers of a file’s size if it is between 1 MB/MiB and 1 GB/GiB

`ng`
: the numbers of a file’s size if it is between 1 GB/GiB and 1 TB/TiB

`nt`
: the numbers of a file’s size if it is 1 TB/TiB or higher

`sb`
: the units of a file’s size (sets `ub`, `uk`, `um`, `ug` and `uh`)

`ub`
: the units of a file’s size if it is lower than 1 KB/Kib

`uk`
: the units of a file’s size if it is between 1 KB/KiB and 1 MB/MiB

`um`
: the units of a file’s size if it is between 1 MB/MiB and 1 GB/GiB

`ug`
: the units of a file’s size if it is between 1 GB/GiB and 1 TB/TiB

`ut`
: the units of a file’s size if it is 1 TB/TiB or higher

`df`
: a device’s major ID

`ds`
: a device’s minor ID

`uu`
: a user that’s you

`un`
: a user that’s someone else

`gu`
: a group that you belong to

`gn`
: a group you aren’t a member of

`lc`
: a number of hard links

`lm`
: a number of hard links for a regular file with at least two

`ga`
: a new flag in Git

`gm`
: a modified flag in Git

`gd`
: a deleted flag in Git

`gv`
: a renamed flag in Git

`gt`
: a modified metadata flag in Git

`xx`
: “punctuation”, including many background UI elements

`da`
: a file’s date

`in`
: a file’s inode number

`bl`
: a file’s number of blocks

`hd`
: the header row of a table

`lp`
: the path of a symlink

`cc`
: an escaped character in a filename

`bO`
: the overlay style for broken symlink paths

Values in `EXA_COLORS` override those given in `LS_COLORS`, so you don’t need to re-write an existing `LS_COLORS` variable with proprietary extensions.


LIST OF STYLES
==============

Unlike some versions of `ls`, the given ANSI values must be valid colour codes: exa won’t just print out whichever characters are given.

The codes accepted by exa are:

`1`
: for bold

`4`
: for underline

`31`
: for red text

`32`
: for green text

`33`
: for yellow text

`34`
: for blue text

`35`
: for purple text

`36`
: for cyan text

`37`
: for white text

`38;5;nnn`
: for a colour from 0 to 255 (replace the `nnn` part)

Many terminals will treat bolded text as a different colour, or at least provide the option to.

exa provides its own built-in set of file extension mappings that cover a large range of common file extensions, including documents, archives, media, and temporary files.
Any mappings in the environment variables will override this default set: running exa with `LS_COLORS="*.zip=32"` will turn zip files green but leave the colours of other compressed files alone.

You can also disable this built-in set entirely by including a `reset` entry at the beginning of `EXA_COLORS`.
So setting `EXA_COLORS="reset:*.txt=31"` will highlight only text files; setting `EXA_COLORS="reset"` will highlight nothing.


AUTHOR
======

exa is maintained by Benjamin ‘ogham’ Sago and many other contributors.

**Website:** `https://the.exa.website/` \
**Source code:** `https://github.com/ogham/exa` \
**Contributors:** `https://github.com/ogham/exa/graphs/contributors`


SEE ALSO
========

- `exa(1)`
