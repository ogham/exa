exa
===

exa is a replacement for `ls` written in Rust.

![Build status](https://travis-ci.org/ogham/exa.svg?branch=master)

Screenshot
----------

![Screenshot of exa](https://raw.githubusercontent.com/ogham/exa/master/screenshot.png)

Options
-------

- **-a**, **--all**: show dot files
- **-b**, **--binary**: use binary (power of two) file sizes
- **-g**, **--group**: show group as well as user
- **-h**, **--header**: show a header row
- **-i**, **--inode**: show inode number column
- **-l**, **--links**: show number of hard links column
- **-r**, **--reverse**: reverse sort order
- **-s**, **--sort=(name, size, ext)**: field to sort by
- **-S**, **--blocks**: show number of file system blocks


Installation
------------

exa is written in [Rust](http://www.rust-lang.org). You should use the nightly, rather than the 0.10 release, which is rather out of date at this point. Once you have Rust set up, a simple `make` should compile exa.
