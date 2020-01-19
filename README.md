# exa [![Build status](https://travis-ci.org/ogham/exa.svg)](https://travis-ci.org/ogham/exa)

[exa](https://the.exa.website/) is a replacement for `ls` written in Rust.

## Rationale

**exa**  is a modern replacement for the command-line program ls that ships with Unix and Linux operating systems, with more features and better defaults. It uses colours to distinguish file types and metadata. It knows about symlinks, extended attributes, and Git. And it’s **small**, **fast**, and just one **single binary**.

By deliberately making some decisions differently, exa attempts to be a more featureful, more user-friendly version of ls.

## Screenshots

![Screenshots of exa](screenshots.png)


## Options

exa’s options are almost, but not quite, entirely unlike `ls`'s.

### Display Options

- **-1**, **--oneline**: display one entry per line
- **-G**, **--grid**: display entries as a grid (default)
- **-l**, **--long**: display extended details and attributes
- **-R**, **--recurse**: recurse into directories
- **-T**, **--tree**: recurse into directories as a tree
- **-x**, **--across**: sort the grid across, rather than downwards
- **--colo[u]r**: when to use terminal colours
- **--colo[u]r-scale**: highlight levels of file sizes distinctly

### Filtering Options

- **-a**, **--all**: show hidden and 'dot' files
- **-d**, **--list-dirs**: list directories like regular files
- **-L**, **--level=(depth)**: limit the depth of recursion
- **-r**, **--reverse**: reverse the sort order
- **-s**, **--sort=(field)**: which field to sort by
- **--group-directories-first**: list directories before other files
- **-D**, **--only-dirs**: list only directories
- **--git-ignore**: ignore files mentioned in `.gitignore`
- **-I**, **--ignore-glob=(globs)**: glob patterns (pipe-separated) of files to ignore

Pass the `--all` option twice to also show the `.` and `..` directories.

### Long View Options

These options are available when running with --long (`-l`):

- **-b**, **--binary**: list file sizes with binary prefixes
- **-B**, **--bytes**: list file sizes in bytes, without any prefixes
- **-g**, **--group**: list each file's group
- **-h**, **--header**: add a header row to each column
- **-H**, **--links**: list each file's number of hard links
- **-i**, **--inode**: list each file's inode number
- **-m**, **--modified**: use the modified timestamp field
- **-S**, **--blocks**: list each file's number of file system blocks
- **-t**, **--time=(field)**: which timestamp field to use
- **-u**, **--accessed**: use the accessed timestamp field
- **-U**, **--created**: use the created timestamp field
- **-@**, **--extended**: list each file's extended attributes and sizes
- **--git**: list each file's Git status, if tracked or ignored
- **--time-style**: how to format timestamps
- **--no-permissions**: suppress the permissions field
- **--no-filesize**: suppress the filesize field
- **--no-user**: suppress the user field
- **--no-time**: suppress the time field

- Valid **--color** options are **always**, **automatic**, and **never**.
- Valid sort fields are **accessed**, **changed**, **created**, **extension**, **Extension**, **inode**, **modified**, **name**, **Name**, **size**, **type**, and **none**. Fields starting with a capital letter sort uppercase before lowercase. The modified field has the aliases **date**, **time**, and **newest**, while its reverse has the aliases **age** and **oldest**.
- Valid time fields are **modified**, **changed**, **accessed**, and **created**.
- Valid time styles are **default**, **iso**, **long-iso**, and **full-iso**.


## Installation

exa is written in [Rust](http://www.rust-lang.org). You will need rustc version 1.35.0 or higher. The recommended way to install Rust is from the official download page.
Once you have it set up, a simple `make install` will compile exa and install it into `/usr/local/bin`.

exa depends on [libgit2](https://github.com/alexcrichton/git2-rs) for certain features.
If you’re unable to compile libgit2, you can opt out of Git support by running `cargo build --release --no-default-features`.

If you intend to compile for musl you will need to use the flag vendored-openssl if you want to get the Git feature working: `cargo build --release --target=x86_64-unknown-linux-musl --features vendored-openssl,git`

### Cargo Install

If you’re using a recent version of Cargo (0.5.0 or higher), you can use the `cargo install` command:

    cargo install exa

or:

    cargo install --no-default-features exa

Cargo will build the `exa` binary and place it in `$HOME/.cargo` (this location can be overridden by setting the `--root` option).

### Homebrew

If you're using [homebrew](https://brew.sh/), you can use the `brew install` command:

    brew install exa

or:

    brew install exa --without-git

[Formulae](https://github.com/Homebrew/homebrew-core/blob/master/Formula/exa.rb)

### Fedora

You can install the `exa` package from the official Fedora repositories by running:

    dnf install exa

### Nix

`exa` is also installable through [the derivation](https://github.com/NixOS/nixpkgs/blob/master/pkgs/tools/misc/exa/default.nix) using the [nix package manager](https://nixos.org/nix/) by running:

    nix-env -i exa

## Testing with Vagrant

exa uses [Vagrant][] to configure virtual machines for testing.

Programs such as exa that are basically interfaces to the system are [notoriously difficult to test][testing]. Although the internal components have unit tests, it’s impossible to do a complete end-to-end test without mandating the current user’s name, the time zone, the locale, and directory structure to test. (And yes, these tests are worth doing. I have missed an edge case on more than one occasion.)

The initial attempt to solve the problem was just to create a directory of “awkward” test cases, run exa on it, and make sure it produced the correct output. But even this output would change if, say, the user’s locale formats dates in a different way. These can be mocked inside the code, but at the cost of making that code more complicated to read and understand.

An alternative solution is to fake *everything*: create a virtual machine with a known state and run the tests on *that*. This is what Vagrant does. Although it takes a while to download and set up, it gives everyone the same development environment to test for any obvious regressions.

[Vagrant]: https://www.vagrantup.com/
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
