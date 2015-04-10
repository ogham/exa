# exa [![Build status](https://travis-ci.org/ogham/exa.svg)](https://travis-ci.org/ogham/exa)

[exa](http://bsago.me/exa) is a replacement for `ls` written in Rust.

**You'll have to use the nightly, rather than Rust beta. Sorry about that.**


## Screenshot

![Screenshot of exa](https://raw.githubusercontent.com/ogham/exa/master/screenshot.png)


## Options

- **-1**, **--oneline**: display one entry per line
- **-a**, **--all**: show dot files
- **-b**, **--binary**: use binary (power of two) file sizes
- **-B**, **--bytes**: list file sizes in bytes, without prefixes
- **-d**, **--list-dirs**: list directories as regular files
- **-g**, **--group**: show group as well as user
- **--group-directories-first**: list directories before other files
- **--git**: show git status (depends on libgit2, see below)
- **-h**, **--header**: show a header row
- **-H**, **--links**: show number of hard links column
- **-i**, **--inode**: show inode number column
- **-l**, **--long**: display extended details and attributes
- **-L**, **--level=(depth)**: maximum depth of recursion
- **-m**, **--modified**: display timestamp of most recent modification
- **-r**, **--reverse**: reverse sort order
- **-R**, **--recurse**: recurse into subdirectories
- **-s**, **--sort=(field)**: field to sort by
- **-S**, **--blocks**: show number of file system blocks
- **-t**, **--time=(field)**: which timestamp to show for a file
- **-T**, **--tree**: recurse into subdirectories in a tree view
- **-u**, **--accessed**: display timestamp of last access for a file
- **-U**, **--created**: display timestamp of creation of a file
- **-x**, **--across**: sort multi-column view entries across
- **-@**, **--extended**: display extended attribute keys and sizes

You can sort by **name**, **size**, **ext**, **inode**, **modified**, **created**, **accessed**, or **none**.


## Installation

exa is written in [Rust](http://www.rust-lang.org). You'll have to use the nightly -- I try to keep it up to date with the latest version when possible.  Once you have it set up, a simple `make install` will compile exa and install it into `/usr/local/bin`.

exa depends on [libgit2](https://github.com/alexcrichton/git2-rs) for certain features. If you're unable to compile libgit2, you can opt out of Git support by passing `--no-default-features` to Cargo.
