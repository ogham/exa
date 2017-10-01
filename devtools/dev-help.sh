# This file prints out some help text that says which commands are available
# in the VM. It gets executed during Vagrant provisioning and its output gets
# dumped into /etc/motd, to print it when a user starts a new Vagrant session.


echo -e "
\033[1;33mThe exa development environment!\033[0m
exa's source is available at \033[33m/vagrant\033[0m.
Binaries get built into \033[33m/home/ubuntu/target\033[0m.

\033[4mCommands\033[0m
\033[32;1mb\033[0m or \033[32;1mbuild-exa\033[0m to run \033[1mcargo build\033[0m
\033[32;1mt\033[0m or \033[32;1mtest-exa\033[0m to run \033[1mcargo test\033[0m
\033[32;1mx\033[0m or \033[32;1mrun-xtests\033[0m to run \033[1m/vagrant/xtests/run.sh\033[0m
\033[32;1mc\033[0m or \033[32;1mcompile-exa\033[0m to run all three
\033[32;1mdebug\033[0m to toggle printing logs
\033[32;1mstrict\033[0m to toggle strict mode
\033[32;1mcolors\033[0m to toggle custom colours
"
