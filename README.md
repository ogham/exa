# exa [![Build status](https://travis-ci.org/ogham/exa.svg)](https://travis-ci.org/ogham/exa)

exa is a replacement for `ls` written in Rust.


## Screenshot

![Screenshot of exa](https://raw.githubusercontent.com/ogham/exa/master/screenshot.png)


## Options

- **-1**, **--oneline**: display one entry per line
- **-a**, **--all**: show dot files
- **-b**, **--binary**: use binary (power of two) file sizes
- **-d**, **--list-dirs**: list directories as regular files
- **-g**, **--group**: show group as well as user
- **-h**, **--header**: show a header row
- **-H**, **--links**: show number of hard links column
- **-i**, **--inode**: show inode number column
- **-l**, **--long**: display extended details and attributes
- **-r**, **--reverse**: reverse sort order
- **-s**, **--sort=(field)**: field to sort by
- **-S**, **--blocks**: show number of file system blocks
- **-x**, **--across**: sort multi-column view entries across

You can sort by **name**, **size**, **ext**, **inode**, or **none**.


## Installation

exa is written in [Rust](http://www.rust-lang.org). You'll have to use the nightly -- I try to keep it up to date with the latest version when possible. Once you have it set up, a simple `cargo build` will pull in all the dependencies and compile exa.
