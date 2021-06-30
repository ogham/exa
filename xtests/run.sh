#!/bin/bash
trap 'exit' ERR

# Check for release mode
case "$1" in
  "--release") exa_binary="$HOME/target/release/exa" ;;
  *)           exa_binary="$HOME/target/debug/exa" ;;
esac

if [ ! -e /vagrant ]; then
  echo "The extended tests must be run on the Vagrant machine."
  exit 1
fi

if [ ! -f "$exa_binary" ]; then
  echo "exa binary ($exa_binary) does not exist"
  if [ "$1" != "--release" ]; then echo -e "create it first with \033[1;32mbuild-exa\033[0m or \033[1;32mb\033[0m"; fi
  exit 1
fi

echo -e "#!/bin/sh\nexec $exa_binary --colour=always \"\$@\"" > /tmp/exa
chmod +x /tmp/exa
export PATH="/tmp:$PATH"

# Unset any environment variables
export EXA_STRICT=""
export EXA_DEBUG=""
export LS_COLORS=""
export EXA_COLORS=""

# Run the tests
exec specsheet $(dirname "$0")/*.toml -O cmd.shell=bash
