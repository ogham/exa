set -e

# This script builds a publishable release-worthy version of exa.
# It gets the version number, builds exa using cargo, tests it, strips the
# binary, compresses it into a zip, then puts it in /vagrant so it’s
# accessible from the host machine.
#
# If you’re in the VM, you can run it using the ‘package-exa’ command.


# First, we need to get the version number to figure out what to call the zip.
# We do this by getting the first line from the Cargo.toml that matches
# /version/, removing its whitespace, and building a command out of it, so the
# shell executes something like `exa_version="0.8.0"`, which it understands as
# a variable definition. Hey, it’s not a hack if it works.
toml_file="/vagrant/Cargo.toml"
eval exa_$(grep version $toml_file | head -n 1 | sed "s/ //g")
if [ -z "$exa_version" ]; then
  echo "Failed to parse version number! Can't build exa!"
  exit
else
  echo "Building exa v$exa_version"
fi

# Compilation is done in --release mode, which takes longer but produces a
# faster binary. This binary gets built to a different place, so the extended
# tests script needs to be told which one to use.
echo -e "\n\033[4mCompiling release version of exa...\033[0m"
exa_linux_binary="/vagrant/exa-linux-x86_64"
rm -vf "$exa_linux_binary"
cargo build --release --manifest-path "$toml_file"
cargo test --release --manifest-path "$toml_file" --lib -- --quiet
/vagrant/xtests/run.sh --release
cp /home/ubuntu/target/release/exa "$exa_linux_binary"

# Stripping the binary before distributing it removes a bunch of debugging
# symbols, saving some space.
echo -e "\n\033[4mStripping binary...\033[0m"
strip -v "$exa_linux_binary"

# Compress the binary for upload. The ‘-j’ flag is necessary to avoid the
# /vagrant path being in the zip too. Only the zip gets the version number, so
# the binaries can have consistent names, but it’s still possible to tell
# different *downloads* apart.
echo -e "\n\033[4mZipping binary...\033[0m"
exa_linux_zip="/vagrant/exa-linux-x86_64-${exa_version}.zip"
rm -vf "$exa_linux_zip"
zip -j "$exa_linux_zip" "$exa_linux_binary"

# There was a problem a while back where a library was getting unknowingly
# *dynamically* linked, which broke the whole ‘self-contained binary’ concept.
# So dump the linker table, in case anything unscrupulous shows up.
echo -e "\n\033[4mLibraries linked:\033[0m"
ldd "$exa_linux_binary" | sed "s/\t//"

# Might as well use it to test itself, right?
echo -e "\n\033[4mAll done! Files produced:\033[0m"
"$exa_linux_binary" "$exa_linux_binary" "$exa_linux_zip" -lB
