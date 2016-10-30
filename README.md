# exa [![Build status](https://travis-ci.org/ogham/exa.svg)](https://travis-ci.org/ogham/exa)

[exa](https://the.exa.website/) is a replacement for `ls` written in Rust.

Works on all recent Rust versions >= 1.4.0.


## Screenshots

![Screenshots of exa](screenshots.png)


## Options

exa’s options are similar, but not exactly the same, as `ls`.

### Display Options

- **-1**, **--oneline**: display one entry per line
- **-G**, **--grid**: display entries in a grid view (default)
- **-l**, **--long**: display extended details and attributes
- **-R**, **--recurse**: recurse into directories
- **-T**, **--tree**: recurse into subdirectories in a tree view
- **-x**, **--across**: sort multi-column view entries across
- **--color**, **--colour**: when to colourise the output

### Filtering Options

- **-a**, **--all**: show dot files
- **-d**, **--list-dirs**: list directories as regular files
- **-L**, **--level=(depth)**: maximum depth of recursion
- **-r**, **--reverse**: reverse sort order
- **-s**, **--sort=(field)**: field to sort by
- **--group-directories-first**: list directories before other files
- **-I**, **--ignore-glob=(globs)**: glob patterns (pipe-separated) of files to ignore

### Long View Options

These options are available when running with --long (`-l`):

- **-b**, **--binary**: use binary (power of two) file sizes
- **-B**, **--bytes**: list file sizes in bytes, without prefixes
- **-g**, **--group**: show group as well as user
- **-h**, **--header**: show a header row
- **-H**, **--links**: show number of hard links column
- **-i**, **--inode**: show inode number column
- **-m**, **--modified**: display timestamp of most recent modification
- **-S**, **--blocks**: show number of file system blocks
- **-t**, **--time=(field)**: which timestamp to show for a file
- **-u**, **--accessed**: display timestamp of last access for a file
- **-U**, **--created**: display timestamp of creation of a file
- **-@**, **--extended**: display extended attribute keys and sizes
- **--git**: show Git status for a file

Accepted **--color** options are **always**, **automatic**, and **never**.
Valid sort fields are **name**, **size**, **extension**, **modified**, **accessed**, **created**, **inode**, and **none**.
Valid time fields are **modified**, **accessed**, and **created**.


## Installation

exa is written in [Rust](http://www.rust-lang.org).
Once you have it set up, a simple `make install` will compile exa and install it into `/usr/local/bin`.

exa depends on [libgit2](https://github.com/alexcrichton/git2-rs) for certain features.
If you’re unable to compile libgit2, you can opt out of Git support by running `cargo build --release --no-default-features`.

### Cargo Install

If you’re using a recent version of Cargo (0.5.0 or higher), you can use the `cargo install` command:

    cargo install --git https://github.com/ogham/exa

or:

    cargo install --no-default-features --git https://github.com/ogham/exa

Cargo will clone the repository to a temporary directory, build it there and place the `exa` binary to: `$HOME/.cargo` (and can be overridden by setting the `--root` option).


## Testing with Vagrant

exa uses [Vagrant][] to configure virtual machines for testing.

Programs such as exa that are basically interfaces to the system are [notoriously difficult to test][testing]. Although the internal components have unit tests, it’s impossible to do a complete end-to-end test without mandating the current user’s name, the time zone, the locale, and directory structure to test. (And yes, these tests are worth doing. I have missed an edge case on more than one occasion.)

The initial attempt to solve the problem was just to create a directory of “awkward” test cases, run exa on it, and make sure it produced the correct output. But even this output would change if, say, the user’s locale formats dates in a different way. These can be mocked inside the code, but at the cost of making that code more complicated to read and understand.

An alternative solution is to fake *everything*: create a virtual machine with a known state and run the tests on *that*. This is what Vagrant does. Although it takes a while to download and set up, it gives everyone the same development environment to test for any obvious regressions.

[Vagrant]: https://www.vagrantup.com/docs/why-vagrant/
[testing]: https://eev.ee/blog/2016/08/22/testing-for-people-who-hate-testing/#troublesome-cases

First, initialise the VM:

    host$ vagrant up

The first command downloads the virtual machine image, and then runs our provisioning script, which installs Rust, exa’s dependencies, configures the environment, and generates some awkward files and folders to use as test cases. This takes some time, but it does write to output occasionally. Once this is done, you can SSH in, and build and test:

    host$ vagrant ssh
    vm$ cd /vagrant
    vm$ cargo build
    vm$ ./xtests/run
    All the tests passed!


### Running without Vagrant

Of course, the drawback of having a standard development environment is that you stop noticing bugs that occur outside of it. For this reason, Vagrant isn’t a *necessary* development step — it’s there if you’d like to use it, but exa still gets used and tested on other platforms. It can still be built and compiled on any target triple that it supports, VM or no VM, with `cargo build` and `cargo test`.
