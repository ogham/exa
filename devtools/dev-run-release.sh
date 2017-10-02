#!/bin/bash
if [[ -f ~/target/release/exa ]]; then
  ~/target/release/exa "$@"
else
  echo -e "Release exa binary does not exist!"
  echo -e "Run \033[32;1mb --release\033[0m or \033[32;1mbuild-exa --release\033[0m to create it"
fi
