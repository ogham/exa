# This file is like the other one, except for the testing VM.
# It also gets dumped into /etc/motd.


echo -e "
\033[1;33mThe exa testing environment!\033[0m
This machine is dependency-free, and can be used to test that
released versions of exa still work on vanilla Linux installs.

\033[4mCommands\033[0m
\033[32;1mcheck-release\033[0m to download and verify released binaries
"
