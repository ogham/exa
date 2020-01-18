# Displays the installed versions of Rust and Cargo.
# This gets run from ‘dev-bash.sh’, which gets run from ‘~/.bash_profile’, so
# the versions gets displayed after the help text for a new Vagrant session.

echo -e "\\033[4mVersions\\033[0m"
rustc --version
cargo --version
echo
