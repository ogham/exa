require 'date'

Vagrant.configure(2) do |config|

  # We use Ubuntu instead of Debian because the image comes with two-way
  # shared folder support by default.
  UBUNTU = 'ubuntu/xenial64'

  # The main VM is the one used for development and testing.
  config.vm.define(:exa, primary: true) do |config|
    config.vm.provider :virtualbox do |v|
      v.name = 'exa'
      v.memory = 1024
      v.cpus = 1
    end

    config.vm.box = UBUNTU
    config.vm.hostname = 'exa'


    # Make sure we know the VM image’s default user name. The ‘cassowary’ user
    # (specified later) is used for most of the test *output*, but we still
    # need to know where the ‘target’ and ‘.cargo’ directories go.
    developer = 'ubuntu'


    # Install the dependencies needed for exa to build, as quietly as
    # apt can do.
    config.vm.provision :shell, privileged: true, inline: <<-EOF
      set -xe
      apt-get update
      apt-get install -qq -o=Dpkg::Use-Pty=0 -y \
        git cmake curl attr libgit2-dev zip \
        fish zsh bash bash-completion
    EOF


    # Guarantee that the timezone is UTC -- some of the tests
    # depend on this (for now).
    config.vm.provision :shell, privileged: true, inline:
      %[timedatectl set-timezone UTC]


    # Install Rust.
    # This is done as vagrant, not root, because it’s vagrant
    # who actually uses it. Sent to /dev/null because the progress
    # bar produces a ton of output.
    config.vm.provision :shell, privileged: false, inline:
      %[hash rustc &>/dev/null || curl -sSf https://static.rust-lang.org/rustup.sh | sh &> /dev/null]


    # Use a different ‘target’ directory on the VM than on the host.
    # By default it just uses the one in /vagrant/target, which can
    # cause problems if it has different permissions than the other
    # directories, or contains object files compiled for the host.
    config.vm.provision :shell, privileged: true, inline: <<-EOF
      echo 'PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/home/#{developer}/.cargo/bin"' > /etc/environment
      echo 'CARGO_TARGET_DIR="/home/#{developer}/target"'                                                     >> /etc/environment
    EOF


    # Create a variety of misc scripts.
    config.vm.provision :shell, privileged: true, inline: <<-EOF
      set -xe

      echo -e "#!/bin/sh\n/home/#{developer}/target/debug/exa \"\\$*\"" > /usr/bin/exa
      echo -e "#!/bin/sh\n/home/#{developer}/target/release/exa \"\\$*\"" > /usr/bin/rexa

      echo -e "#!/bin/sh\ncargo build --manifest-path /vagrant/Cargo.toml" > /usr/bin/build-exa
      ln -sf /usr/bin/build-exa /usr/bin/b

      echo -e "#!/bin/sh\ncargo test --manifest-path /vagrant/Cargo.toml --lib -- --quiet" > /usr/bin/test-exa
      ln -sf /usr/bin/test-exa /usr/bin/t

      echo -e "#!/bin/sh\n/vagrant/xtests/run.sh" > /usr/bin/run-xtests
      ln -sf /usr/bin/run-xtests /usr/bin/x

      echo -e "#!/bin/sh\nbuild-exa && test-exa && run-xtests" > /usr/bin/compile-exa
      ln -sf /usr/bin/compile-exa /usr/bin/c

      echo "#!/bin/bash"                                                      > /usr/bin/package-exa
      echo "set -e"                                                          >> /usr/bin/package-exa

      echo 'echo -e "\nCompiling release version of exa..."'                 >> /usr/bin/package-exa
      echo "cargo build --release --manifest-path /vagrant/Cargo.toml"       >> /usr/bin/package-exa
      echo "cargo test --release --manifest-path /vagrant/Cargo.toml --lib"  >> /usr/bin/package-exa
      echo "/vagrant/xtests/run.sh --release"                                >> /usr/bin/package-exa
      echo "cp /home/ubuntu/target/release/exa /vagrant/exa-linux-x86_64"    >> /usr/bin/package-exa

      echo 'echo -e "\nStripping binary..."'                                 >> /usr/bin/package-exa
      echo "strip /vagrant/exa-linux-x86_64"                                 >> /usr/bin/package-exa

      echo 'echo -e "\nZipping binary..."'                                   >> /usr/bin/package-exa
      echo "rm -f /vagrant/exa-linux-x86_64.zip"                             >> /usr/bin/package-exa
      echo "zip -j /vagrant/exa-linux-x86_64.zip /vagrant/exa-linux-x86_64"  >> /usr/bin/package-exa

      echo 'echo -e "\nLibraries linked:"'                                   >> /usr/bin/package-exa
      echo "ldd /vagrant/exa-linux-x86_64"                                   >> /usr/bin/package-exa

      echo 'echo -e "\nAll done!"'                                           >> /usr/bin/package-exa
      echo '/vagrant/exa-linux-x86_64 /vagrant/exa-linux-x86_64* -lB'        >> /usr/bin/package-exa

      chmod +x /usr/bin/{exa,rexa,b,t,x,c,build-exa,test-exa,run-xtests,compile-exa,package-exa}
    EOF


    # Download my patched version of git2-rs.
    # This is basically a hack and we should get rid of it as soon as
    # a better solution comes along.
    # See https://github.com/ogham/exa/issues/194
    config.vm.provision :shell, privileged: false, inline: <<-EOF
      set -xe

      if [ -d /home/ubuntu/git2-rs ]; then
          cd /home/ubuntu/git2-rs
          git pull https://github.com/ogham/git2-rs.git || echo "Failed to update git2-rs fork"
      else
          git clone https://github.com/ogham/git2-rs.git /home/ubuntu/git2-rs
      fi

      mkdir -p /home/ubuntu/.cargo
      echo 'paths = ["/home/ubuntu/git2-rs/libgit2-sys"]' > /home/ubuntu/.cargo/config
    EOF

    # This fix is applied by changing the VM rather than changing the
    # Cargo.toml file so it works for everyone because it’s such a niche
    # build issue, it’s not worth specifying a non-crates.io dependency
    # and losing the ability to `cargo publish` the exa crate there!
    # It also isolates the hackiness to the one place I can test it
    # actually works.


    # Write some welcoming text.
    config.vm.provision :shell, privileged: true, inline: <<-EOF
      rm -f /etc/update-motd.d/*

      echo -e ""                        > /etc/motd
      echo -e "\033[1;33mThe exa development environment!\033[0m"  >> /etc/motd
      echo -e "exa's source is available at \033[33m/vagrant\033[0m."  >> /etc/motd
      echo -e "Binaries get built into \033[33m/home/ubuntu/target\033[0m."  >> /etc/motd
      echo -e ""                        >> /etc/motd
      echo -e "\033[4mCommands\033[0m"  >> /etc/motd
      echo -e "\033[32;1mb\033[0m or \033[32;1mbuild-exa\033[0m to run \033[1mcargo build\033[0m"  >> /etc/motd
      echo -e "\033[32;1mt\033[0m or \033[32;1mtest-exa\033[0m to run \033[1mcargo test\033[0m"   >> /etc/motd
      echo -e "\033[32;1mx\033[0m or \033[32;1mrun-xtests\033[0m to run \033[1m/vagrant/xtests/run.sh\033[0m"  >> /etc/motd
      echo -e "\033[32;1mc\033[0m or \033[32;1mcompile-exa\033[0m to run all three"  >> /etc/motd
      echo -e "\033[32;1mdebug\033[0m to toggle printing logs"  >> /etc/motd
      echo -e "\033[32;1mstrict\033[0m to toggle strict mode"  >> /etc/motd
      echo -e "\033[32;1mcolors\033[0m to toggle custom colours\n"  >> /etc/motd

      # help banner
      echo 'echo -e "\\033[4mVersions\\033[0m"' > /home/ubuntu/.bash_profile
      echo "rustc --version" >> /home/ubuntu/.bash_profile
      echo "cargo --version" >> /home/ubuntu/.bash_profile
      echo "echo" >> /home/ubuntu/.bash_profile

      # cool prompt
      echo 'function nonzero_return() { RETVAL=$?; [ $RETVAL -ne 0 ] && echo "$RETVAL "; }' >> /home/ubuntu/.bash_profile
      echo 'function debug_mode()  { [ -n "$EXA_DEBUG" ]  && echo "debug "; }'  >> /home/ubuntu/.bash_profile
      echo 'function strict_mode() { [ -n "$EXA_STRICT" ] && echo "strict "; }' >> /home/ubuntu/.bash_profile
      echo 'function lsc_mode()    { [ -n "$LS_COLORS" ]  && echo "lsc "; }'    >> /home/ubuntu/.bash_profile
      echo 'function exac_mode()   { [ -n "$EXA_COLORS" ] && echo "exac "; }'   >> /home/ubuntu/.bash_profile
      echo 'export PS1="\\[\\e[1;36m\\]\\h \\[\\e[32m\\]\\w \\[\\e[31m\\]\\`nonzero_return\\`\\[\\e[35m\\]\\`debug_mode\\`\\[\\e[32m\\]\\`lsc_mode\\`\\[\\e[1;32m\\]\\`exac_mode\\`\\[\\e[33m\\]\\`strict_mode\\`\\[\\e[36m\\]\\\\$\\[\\e[0m\\] "' >> /home/ubuntu/.bash_profile

      # environment setting
      echo 'function debug () {' >> /home/ubuntu/.bash_profile
      echo '  case "$1" in "on") export EXA_DEBUG=1 ;;' >> /home/ubuntu/.bash_profile
      echo '    "off") export EXA_DEBUG= ;;' >> /home/ubuntu/.bash_profile
      echo '    "")    [ -n "$EXA_DEBUG" ] && echo "debug on" || echo "debug off" ;;' >> /home/ubuntu/.bash_profile
      echo '    *)     echo "Usage: debug on|off"; return 1 ;; esac; }' >> /home/ubuntu/.bash_profile

      echo 'function strict () {' >> /home/ubuntu/.bash_profile
      echo '  case "$1" in "on") export EXA_STRICT=1 ;;' >> /home/ubuntu/.bash_profile
      echo '    "off") export EXA_STRICT= ;;' >> /home/ubuntu/.bash_profile
      echo '    "") [ -n "$EXA_STRICT" ] && echo "strict on" || echo "strict off" ;;' >> /home/ubuntu/.bash_profile
      echo '    *) echo "Usage: strict on|off"; return 1 ;; esac; }' >> /home/ubuntu/.bash_profile

      echo 'function colors () {' >> /home/ubuntu/.bash_profile
      echo '  case "$1" in ' >> /home/ubuntu/.bash_profile
      echo '    "ls")' >> /home/ubuntu/.bash_profile
      echo '      export LS_COLORS="di=34:ln=35:so=32:pi=33:ex=31:bd=34;46:cd=34;43:su=30;41:sg=30;46:tw=30;42:ow=30;43"'  >> /home/ubuntu/.bash_profile
      echo '      export EXA_COLORS="" ;;' >> /home/ubuntu/.bash_profile
      echo '    "hacker")' >> /home/ubuntu/.bash_profile
      echo '      export LS_COLORS="di=32:ex=32:fi=32:pi=32:so=32:bd=32:cd=32:ln=32:or=32:mi=32"' >> /home/ubuntu/.bash_profile
      echo '      export EXA_COLORS="ur=32:uw=32:ux=32:ue=32:gr=32:gw=32:gx=32:tr=32:tw=32:tx=32:su=32:sf=32:xa=32:sn=32:sb=32:df=32:ds=32:uu=32:un=32:gu=32:gn=32:lc=32:lm=32:ga=32:gm=32:gd=32:gv=32:gt=32:xx=32:da=32:in=32:bl=32:hd=32:lp=32:cc=32:" ;;' >> /home/ubuntu/.bash_profile
      echo '    "off")' >> /home/ubuntu/.bash_profile
      echo '      export LS_COLORS=' >> /home/ubuntu/.bash_profile
      echo '      export EXA_COLORS= ;;' >> /home/ubuntu/.bash_profile
      echo '    "")' >> /home/ubuntu/.bash_profile
      echo '      [ -n "$LS_COLORS" ]  && echo "LS_COLORS=$LS_COLORS"   || echo "ls-colors off"'     >> /home/ubuntu/.bash_profile
      echo '      [ -n "$EXA_COLORS" ] && echo "EXA_COLORS=$EXA_COLORS" || echo "exa-colors off" ;;' >> /home/ubuntu/.bash_profile
      echo '    *) echo "Usage: ls-colors ls|hacker|off"; return 1 ;; esac; }' >> /home/ubuntu/.bash_profile

      # Disable last login date in sshd
      sed -i '/PrintLastLog yes/c\PrintLastLog no' /etc/ssh/sshd_config
      systemctl restart sshd
    EOF


    # Link the completion files so they’re “installed”.
    config.vm.provision :shell, privileged: true, inline: <<-EOF
      set -xe

      test -h /etc/bash_completion.d/exa \
        || ln -s /vagrant/contrib/completions.bash /etc/bash_completion.d/exa

      test -h /usr/share/zsh/vendor-completions/_exa \
        || ln -s /vagrant/contrib/completions.zsh /usr/share/zsh/vendor-completions/_exa

      test -h /usr/share/fish/completions/exa.fish \
        || ln -s /vagrant/contrib/completions.fish /usr/share/fish/completions/exa.fish
    EOF


    # We create two users that own the test files.
    # The first one just owns the ordinary ones, because we don’t want the
    # test outputs to depend on “vagrant” or “ubuntu” existing.
    user = "cassowary"
    config.vm.provision :shell, privileged: true, inline:
      %[id -u #{user} &>/dev/null || useradd #{user}]


    # The second one has a long name, to test that the file owner column
    # widens correctly. The benefit of Vagrant is that we don’t need to
    # set this up on the *actual* system!
    longuser = "antidisestablishmentarienism"
    config.vm.provision :shell, privileged: true, inline:
      %[id -u #{longuser} &>/dev/null || useradd #{longuser}]


    # Because the timestamps are formatted differently depending on whether
    # they’re in the current year or not (see `details.rs`), we have to make
    # sure that the files are created in the current year, so they get shown
    # in the format we expect.
    current_year = Date.today.year
    some_date = "#{current_year}01011234.56"  # 1st January, 12:34:56


    # We also need an UID and a GID that are guaranteed to not exist, to
    # test what happen when they don’t.
    invalid_uid = 666
    invalid_gid = 616


    # Delete old testcases if they exist already, then create a
    # directory to house new ones.
    test_dir = "/testcases"
    config.vm.provision :shell, privileged: true, inline: <<-EOF
      set -xe
      rm -rfv #{test_dir}
      mkdir #{test_dir}
      chmod 777 #{test_dir}
    EOF


    # Awkward file size testcases.
    # This needs sudo to set the files’ users at the very end.
    config.vm.provision :shell, privileged: false, inline: <<-EOF
      set -xe
      mkdir "#{test_dir}/files"
      for i in {1..13}; do
        fallocate -l "$i" "#{test_dir}/files/$i"_bytes
        fallocate -l "$i"KiB "#{test_dir}/files/$i"_KiB
        fallocate -l "$i"MiB "#{test_dir}/files/$i"_MiB
      done

      touch -t #{some_date} "#{test_dir}/files/"*
      chmod 644 "#{test_dir}/files/"*
      sudo chown #{user}:#{user} "#{test_dir}/files/"*
    EOF


    # File name extension testcases.
    # These aren’t tested in details view, but we set timestamps on them to
    # test that various sort options work.
    config.vm.provision :shell, privileged: false, inline: <<-EOF
      set -xe
      mkdir "#{test_dir}/file-names-exts"

      touch "#{test_dir}/file-names-exts/Makefile"

      touch "#{test_dir}/file-names-exts/IMAGE.PNG"
      touch "#{test_dir}/file-names-exts/image.svg"

      touch "#{test_dir}/file-names-exts/VIDEO.AVI"
      touch "#{test_dir}/file-names-exts/video.wmv"

      touch "#{test_dir}/file-names-exts/music.mp3"
      touch "#{test_dir}/file-names-exts/MUSIC.OGG"

      touch "#{test_dir}/file-names-exts/lossless.flac"
      touch "#{test_dir}/file-names-exts/lossless.wav"

      touch "#{test_dir}/file-names-exts/crypto.asc"
      touch "#{test_dir}/file-names-exts/crypto.signature"

      touch "#{test_dir}/file-names-exts/document.pdf"
      touch "#{test_dir}/file-names-exts/DOCUMENT.XLSX"

      touch "#{test_dir}/file-names-exts/COMPRESSED.ZIP"
      touch "#{test_dir}/file-names-exts/compressed.tar.gz"
      touch "#{test_dir}/file-names-exts/compressed.tgz"
      touch "#{test_dir}/file-names-exts/compressed.tar.xz"
      touch "#{test_dir}/file-names-exts/compressed.txz"
      touch "#{test_dir}/file-names-exts/compressed.deb"

      touch "#{test_dir}/file-names-exts/backup~"
      touch "#{test_dir}/file-names-exts/#SAVEFILE#"
      touch "#{test_dir}/file-names-exts/file.tmp"

      touch "#{test_dir}/file-names-exts/compiled.class"
      touch "#{test_dir}/file-names-exts/compiled.o"
      touch "#{test_dir}/file-names-exts/compiled.js"
      touch "#{test_dir}/file-names-exts/compiled.coffee"
    EOF


    # File name testcases.
    # bash really doesn’t want you to create a file with escaped characters
    # in its name, so we have to resort to the echo builtin and touch!
    #
    # The double backslashes are not strictly necessary; without them, Ruby
    # will interpolate them instead of bash, but because Vagrant prints out
    # each command it runs, your *own* terminal will go “ding” from the alarm!
    config.vm.provision :shell, privileged: false, inline: <<-EOF
      set -xe
      mkdir "#{test_dir}/file-names"

      echo -ne "#{test_dir}/file-names/ascii: hello" | xargs -0 touch
      echo -ne "#{test_dir}/file-names/emoji: [🆒]"  | xargs -0 touch
      echo -ne "#{test_dir}/file-names/utf-8: pâté"  | xargs -0 touch

      echo -ne "#{test_dir}/file-names/bell: [\\a]"         | xargs -0 touch
      echo -ne "#{test_dir}/file-names/backspace: [\\b]"    | xargs -0 touch
      echo -ne "#{test_dir}/file-names/form-feed: [\\f]"    | xargs -0 touch
      echo -ne "#{test_dir}/file-names/new-line: [\\n]"     | xargs -0 touch
      echo -ne "#{test_dir}/file-names/return: [\\r]"       | xargs -0 touch
      echo -ne "#{test_dir}/file-names/tab: [\\t]"          | xargs -0 touch
      echo -ne "#{test_dir}/file-names/vertical-tab: [\\v]" | xargs -0 touch

      echo -ne "#{test_dir}/file-names/escape: [\\033]"               | xargs -0 touch
      echo -ne "#{test_dir}/file-names/ansi: [\\033[34mblue\\033[0m]" | xargs -0 touch

      echo -ne "#{test_dir}/file-names/invalid-utf8-1: [\\xFF]"                | xargs -0 touch
      echo -ne "#{test_dir}/file-names/invalid-utf8-2: [\\xc3\\x28]"           | xargs -0 touch
      echo -ne "#{test_dir}/file-names/invalid-utf8-3: [\\xe2\\x82\\x28]"      | xargs -0 touch
      echo -ne "#{test_dir}/file-names/invalid-utf8-4: [\\xf0\\x28\\x8c\\x28]" | xargs -0 touch

      echo -ne "#{test_dir}/file-names/new-line-dir: [\\n]"                | xargs -0 mkdir
      echo -ne "#{test_dir}/file-names/new-line-dir: [\\n]/subfile"        | xargs -0 touch
      echo -ne "#{test_dir}/file-names/new-line-dir: [\\n]/another: [\\n]" | xargs -0 touch
      echo -ne "#{test_dir}/file-names/new-line-dir: [\\n]/broken"         | xargs -0 touch

      mkdir "#{test_dir}/file-names/links"
      ln -s "#{test_dir}/file-names/new-line-dir"*/* "#{test_dir}/file-names/links"

      echo -ne "#{test_dir}/file-names/new-line-dir: [\\n]/broken" | xargs -0 rm
    EOF


    # Special file testcases.
    config.vm.provision :shell, privileged: false, inline: <<-EOF
      set -xe
      mkdir "#{test_dir}/specials"

      sudo mknod "#{test_dir}/specials/block-device" b  3 60
      sudo mknod "#{test_dir}/specials/char-device"  c 14 40
      sudo mknod "#{test_dir}/specials/named-pipe"   p

      sudo touch -t #{some_date} "#{test_dir}/specials/"*
    EOF


    # Awkward symlink testcases.
    config.vm.provision :shell, privileged: false, inline: <<-EOF
      set -xe
      mkdir "#{test_dir}/links"

      ln -s /            "#{test_dir}/links/root"
      ln -s /usr         "#{test_dir}/links/usr"
      ln -s nowhere      "#{test_dir}/links/broken"
      ln -s /proc/1/root "#{test_dir}/links/forbidden"

      touch "#{test_dir}/links/some_file"
      ln -s "#{test_dir}/links/some_file" "#{test_dir}/links/some_file_absolute"
      (cd "#{test_dir}/links"; ln -s "some_file" "some_file_relative")
      (cd "#{test_dir}/links"; ln -s "."         "current_dir")
      (cd "#{test_dir}/links"; ln -s ".."        "parent_dir")
      (cd "#{test_dir}/links"; ln -s "itself"    "itself")
    EOF


    # Awkward passwd testcases.
    # sudo is needed for these because we technically aren’t a member
    # of the groups (because they don’t exist), and chown and chgrp
    # are smart enough to disallow it!
    config.vm.provision :shell, privileged: false, inline: <<-EOF
      set -xe
      mkdir "#{test_dir}/passwd"

      touch -t #{some_date}             "#{test_dir}/passwd/unknown-uid"
      chmod 644                         "#{test_dir}/passwd/unknown-uid"
      sudo chown #{invalid_uid}:#{user} "#{test_dir}/passwd/unknown-uid"

      touch -t #{some_date}             "#{test_dir}/passwd/unknown-gid"
      chmod 644                         "#{test_dir}/passwd/unknown-gid"
      sudo chown #{user}:#{invalid_gid} "#{test_dir}/passwd/unknown-gid"
    EOF


    # Awkward permission testcases.
    # Differences in the way ‘chmod’ handles setting ‘setuid’ and ‘setgid’
    # when you don’t already own the file mean that we need to use ‘sudo’
    # to change permissions to those.
    config.vm.provision :shell, privileged: false, inline: <<-EOF
      set -xe
      mkdir "#{test_dir}/permissions"

      mkdir                      "#{test_dir}/permissions/forbidden-directory"
      chmod 000                  "#{test_dir}/permissions/forbidden-directory"
      touch -t #{some_date}      "#{test_dir}/permissions/forbidden-directory"
      sudo chown #{user}:#{user} "#{test_dir}/permissions/forbidden-directory"

      for perms in 000 001 002 004 010 020 040 100 200 400 644 755 777 1000 1001 2000 2010 4000 4100 7666 7777; do
        touch                      "#{test_dir}/permissions/$perms"
        sudo chown #{user}:#{user} "#{test_dir}/permissions/$perms"
        sudo chmod $perms          "#{test_dir}/permissions/$perms"
        sudo touch -t #{some_date} "#{test_dir}/permissions/$perms"
      done
    EOF

    old = '200303030000.00'
    med = '200606152314.29'   # the june gets used for fr_FR locale tests
    new = '200912221038.53'   # and the december for ja_JP local tests

    # Awkward date and time testcases.
    config.vm.provision :shell, privileged: false, inline: <<-EOF
      set -xe
      mkdir "#{test_dir}/dates"

      # there's no way to touch the created date of a file...
      # so we have to do this the old-fashioned way!
      # (and make sure these don't actually get listed)
      touch -t #{old}    "#{test_dir}/dates/peach";  sleep 1
      touch -t #{med}    "#{test_dir}/dates/plum";   sleep 1
      touch -t #{new}    "#{test_dir}/dates/pear"

      # modified dates
      touch -t #{old} -m "#{test_dir}/dates/pear"
      touch -t #{med} -m "#{test_dir}/dates/peach"
      touch -t #{new} -m "#{test_dir}/dates/plum"

      # accessed dates
      touch -t #{old} -a "#{test_dir}/dates/plum"
      touch -t #{med} -a "#{test_dir}/dates/pear"
      touch -t #{new} -a "#{test_dir}/dates/peach"

      sudo chown #{user}:#{user} -R "#{test_dir}/dates"
    EOF


    # Awkward extended attribute testcases.
    # We need to test combinations of various numbers of files *and*
    # extended attributes in directories. Turns out, the easiest way to
    # do this is to generate all combinations of files with “one-xattr”
    # or “two-xattrs” in their name and directories with “empty” or
    # “one-file” in their name, then just give the right number of
    # xattrs and children to those.
    config.vm.provision :shell, privileged: false, inline: <<-EOF
      set -xe
      mkdir "#{test_dir}/attributes"

      mkdir "#{test_dir}/attributes/files"
      touch "#{test_dir}/attributes/files/"{no-xattrs,one-xattr,two-xattrs}{,_forbidden}

      mkdir "#{test_dir}/attributes/dirs"
      mkdir "#{test_dir}/attributes/dirs/"{no-xattrs,one-xattr,two-xattrs}_{empty,one-file,two-files}{,_forbidden}

      setfattr -n user.greeting         -v hello "#{test_dir}/attributes"/**/*{one-xattr,two-xattrs}*
      setfattr -n user.another_greeting -v hi    "#{test_dir}/attributes"/**/*two-xattrs*

      for dir in "#{test_dir}/attributes/dirs/"*one-file*; do
        touch $dir/file-in-question
      done

      for dir in "#{test_dir}/attributes/dirs/"*two-files*; do
        touch $dir/this-file
        touch $dir/that-file
      done

      find "#{test_dir}/attributes" -exec touch {} -t #{some_date} \\;

      # I want to use the following to test,
      # but it only works on macos:
      #chmod +a "#{user} deny readextattr" "#{test_dir}/attributes"/**/*_forbidden

      sudo chmod 000                "#{test_dir}/attributes"/**/*_forbidden
      sudo chown #{user}:#{user} -R "#{test_dir}/attributes"
    EOF


    # A sample Git repository
    # This uses cd because it's easier than telling Git where to go each time
    config.vm.provision :shell, privileged: false, inline: <<-EOF
      set -xe
      mkdir "#{test_dir}/git"
      cd    "#{test_dir}/git"
      git init

      mkdir edits additions moves

      echo "original content" | tee edits/{staged,unstaged,both}
      echo "this file gets moved" > moves/hither

      git add edits moves
      git commit -m "Automated test commit"


      echo "modifications!" | tee edits/{staged,both}
      touch additions/{staged,edited}
      mv moves/{hither,thither}

      git add edits moves additions
      echo "more modifications!" | tee edits/unstaged edits/both additions/edited
      touch additions/unstaged

      find "#{test_dir}/git" -exec touch {} -t #{some_date} \\;
      sudo chown #{user}:#{user} -R "#{test_dir}/git"
    EOF


    # A second Git repository
    # for testing two at once
    config.vm.provision :shell, privileged: false, inline: <<-EOF
      set -xe
      mkdir -p "#{test_dir}/git2/deeply/nested/directory"
      cd       "#{test_dir}/git2"
      git init

      touch "deeply/nested/directory/upd8d"
      git add "deeply/nested/directory/upd8d"
      git commit -m "Automated test commit"

      echo "Now with contents" > "deeply/nested/directory/upd8d"
      touch "deeply/nested/directory/l8st"

      echo -e "target\n*.mp3" > ".gitignore"
      mkdir "ignoreds"
      touch "ignoreds/music.mp3"
      touch "ignoreds/music.m4a"
      mkdir "ignoreds/nested"
      touch "ignoreds/nested/70s grove.mp3"
      touch "ignoreds/nested/funky chicken.m4a"

      mkdir "target"
      touch "target/another ignored file"

      mkdir "deeply/nested/repository"
      cd    "deeply/nested/repository"
      git init
      touch subfile

      find "#{test_dir}/git2" -exec touch {} -t #{some_date} \\;
      sudo chown #{user}:#{user} -R "#{test_dir}/git2"
    EOF

    # Hidden and dot file testcases.
    # We need to set the permissions of `.` and `..` because they actually
    # get displayed in the output here, so this has to come last.
    config.vm.provision :shell, privileged: false, inline: <<-EOF
      set -xe
      shopt -u dotglob
      GLOBIGNORE=".:.."

      mkdir "#{test_dir}/hiddens"
      touch "#{test_dir}/hiddens/visible"
      touch "#{test_dir}/hiddens/.hidden"
      touch "#{test_dir}/hiddens/..extra-hidden"

      # ./hiddens/
      touch -t #{some_date}      "#{test_dir}/hiddens/"*
      chmod 644                  "#{test_dir}/hiddens/"*
      sudo chown #{user}:#{user} "#{test_dir}/hiddens/"*

      # .
      touch -t #{some_date} "#{test_dir}/hiddens"
      chmod 755 "#{test_dir}/hiddens"
      sudo chown #{user}:#{user} "#{test_dir}/hiddens"

      # ..
      sudo touch -t #{some_date} "#{test_dir}"
      sudo chmod 755 "#{test_dir}"
      sudo chown #{user}:#{user} "#{test_dir}"
    EOF


    # Set up some locales
    config.vm.provision :shell, privileged: false, inline: <<-EOF
      set -xe

      # uncomment these from the config file
      sudo sed -i '/fr_FR.UTF-8/s/^# //g' /etc/locale.gen
      sudo sed -i '/ja_JP.UTF-8/s/^# //g' /etc/locale.gen
      sudo locale-gen
    EOF


    # Install kcov for test coverage
    # This doesn’t run coverage over the xtests so it’s less useful for now
    if ENV.key?('INSTALL_KCOV')
      config.vm.provision :shell, privileged: false, inline: <<-EOF
        set -xe

        test -e ~/.cargo/bin/cargo-kcov \
          || cargo install cargo-kcov

        sudo apt-get install -qq -o=Dpkg::Use-Pty=0 -y \
          cmake g++ pkg-config \
          libcurl4-openssl-dev libdw-dev binutils-dev libiberty-dev

        cargo kcov --print-install-kcov-sh | sudo sh
      EOF
    end
  end


  # Remember that problem that exa had where the binary wasn’t actually
  # self-contained? Or the problem where the Linux binary was actually the
  # macOS binary in disguise?
  #
  # This is a “fresh” VM that intentionally downloads no dependencies and
  # installs nothing so that we can check that exa still runs!
  config.vm.define(:fresh) do |config|
    config.vm.box = UBUNTU
    config.vm.hostname = 'fresh'

    config.vm.provider :virtualbox do |v|
      v.name = 'exa-fresh'
      v.memory = 384
      v.cpus = 1
    end
  end
end
