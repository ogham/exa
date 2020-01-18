set -e

# This script builds a publishable release-worthy version of exa.
# It gets the version number, builds exa using cargo, tests it, strips the
# binary, and compresses it into a zip.
#
# It’s *mostly* the same as dev-package-for-linux.sh, except with some
# Mach-specific things (otool instead of ldd), BSD-coreutils-specific things,
# and it doesn’t run the xtests.


# Virtualising macOS is a legal minefield, so this script is ‘local’ instead
# of ‘dev’: I run it from my actual machine, rather than from a VM.
uname=`uname -s`
if [[ "$uname" != "Darwin" ]]; then
  echo "Gotta be on Darwin to run this (detected '$uname')!"
  exit 1
fi

# First, we need to get the version number to figure out what to call the zip.
# We do this by getting the first line from the Cargo.toml that matches
# /version/, removing its whitespace, and building a command out of it, so the
# shell executes something like `exa_version="0.8.0"`, which it understands as
# a variable definition. Hey, it’s not a hack if it works.
#
# Because this can’t use the absolute /vagrant path, this has to use what this
# SO answer calls a “quoting disaster”: https://stackoverflow.com/a/20196098/3484614
# You will also need GNU coreutils: https://stackoverflow.com/a/4031502/3484614
exa_root="$(dirname "$(dirname "$(greadlink -fm "$0")")")"
toml_file="$exa_root"/Cargo.toml
eval exa_$(grep version $toml_file | head -n 1 | sed "s/ //g")
if [ -z "$exa_version" ]; then
  echo "Failed to parse version number! Can't build exa!"
  exit 1
fi

# Weekly builds have a bit more information in their version number (see build.rs).
if [[ "$1" == "--weekly" ]]; then
  git_hash=`GIT_DIR=$exa_root/.git git rev-parse --short --verify HEAD`
  date=`date +"%Y-%m-%d"`
  echo "Building exa weekly v$exa_version, date $date, Git hash $git_hash"
else
  echo "Building exa v$exa_version"
fi

# Compilation is done in --release mode, which takes longer but produces a
# faster binary.
echo -e "\n\033[4mCompiling release version of exa...\033[0m"
exa_macos_binary="$exa_root/exa-macos-x86_64"
rm -vf "$exa_macos_binary" | sed 's/^/removing /'
cargo build --release --manifest-path "$toml_file"
cargo test --release --manifest-path "$toml_file" --lib -- --quiet
# we can’t run the xtests outside the VM!
#/vagrant/xtests/run.sh --release
cp "$exa_root"/target/release/exa "$exa_macos_binary"

# Stripping the binary before distributing it removes a bunch of debugging
# symbols, saving some space.
echo -e "\n\033[4mStripping binary...\033[0m"
strip "$exa_macos_binary"
echo "strip $exa_macos_binary"

# Compress the binary for upload. The ‘-j’ flag is necessary to avoid the
# current path being in the zip too. Only the zip gets the version number, so
# the binaries can have consistent names, and it’s still possible to tell
# different *downloads* apart.
echo -e "\n\033[4mZipping binary...\033[0m"
if [[ "$1" == "--weekly" ]]
  then exa_macos_zip="$exa_root/exa-macos-x86_64-${exa_version}-${date}-${git_hash}.zip"
  else exa_macos_zip="$exa_root/exa-macos-x86_64-${exa_version}.zip"
fi
rm -vf "$exa_macos_zip" | sed 's/^/removing /'
zip -j "$exa_macos_zip" "$exa_macos_binary"

# There was a problem a while back where a library was getting unknowingly
# *dynamically* linked, which broke the whole ‘self-contained binary’ concept.
# So dump the linker table, in case anything unscrupulous shows up.
echo -e "\n\033[4mLibraries linked:\033[0m"
otool -L "$exa_macos_binary" | sed 's/^[[:space:]]*//'

# Might as well use it to test itself, right?
echo -e "\n\033[4mAll done! Files produced:\033[0m"
"$exa_macos_binary" "$exa_macos_binary" "$exa_macos_zip" -lB
