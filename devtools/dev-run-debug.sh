#!/bin/bash
if [[ -f ~/target/debug/exa ]]; then
  ~/target/debug/exa "$@"
else
  echo -e "Debug exa binary does not exist!"
  echo -e "Run \033[32;1mb\033[0m or \033[32;1mbuild-exa\033[0m to create it"
fi
