Vagrant.configure(2) do |config|
    config.vm.provider :virtualbox do |v|
        v.name = 'exa'
        v.memory = 1024
        v.cpus = 1
    end


    # We use Ubuntu instead of Debian because the image comes with two-way
    # shared folder support by default.
    config.vm.box = 'ubuntu/xenial64'
    config.vm.hostname = 'exa'


    # Install the dependencies needed for exa to build.
    config.vm.provision :shell, privileged: true, inline:
        %[apt-get install -y git cmake libssl-dev libgit2-dev libssh2-1-dev curl attr pkg-config]


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
    config.vm.provision :shell, privileged: false, inline:
        %[echo "export CARGO_TARGET_DIR=/home/ubuntu/target" >> ~/.bashrc]


    # We create two users that own the test files.
    # The first one just owns the ordinary ones, because we don’t want to
    # depend on “vagrant” or “ubuntu” existing.
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
    # These are tested in grid view, so we don’t need to bother setting
    # owners or timestamps or anything.
    config.vm.provision :shell, privileged: false, inline: <<-EOF
        set -xe
        mkdir "#{test_dir}/file-names-exts"

        touch "#{test_dir}/file-names-exts/Makefile"

        touch "#{test_dir}/file-names-exts/image.png"
        touch "#{test_dir}/file-names-exts/image.svg"

        touch "#{test_dir}/file-names-exts/video.avi"
        touch "#{test_dir}/file-names-exts/video.wmv"

        touch "#{test_dir}/file-names-exts/music.mp3"
        touch "#{test_dir}/file-names-exts/music.ogg"

        touch "#{test_dir}/file-names-exts/lossless.flac"
        touch "#{test_dir}/file-names-exts/lossless.wav"

        touch "#{test_dir}/file-names-exts/crypto.asc"
        touch "#{test_dir}/file-names-exts/crypto.signature"

        touch "#{test_dir}/file-names-exts/document.pdf"
        touch "#{test_dir}/file-names-exts/document.xlsx"

        touch "#{test_dir}/file-names-exts/compressed.zip"
        touch "#{test_dir}/file-names-exts/compressed.tar.gz"
        touch "#{test_dir}/file-names-exts/compressed.tgz"

        touch "#{test_dir}/file-names-exts/backup~"
        touch "#{test_dir}/file-names-exts/#SAVEFILE#"
        touch "#{test_dir}/file-names-exts/file.tmp"

        touch "#{test_dir}/file-names-exts/compiled.class"
        touch "#{test_dir}/file-names-exts/compiled.o"
        touch "#{test_dir}/file-names-exts/compiled.js"
        touch "#{test_dir}/file-names-exts/compiled.coffee"
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
        ln -s / "#{test_dir}/links/root"
        ln -s /usr "#{test_dir}/links/usr"
        ln -s nowhere "#{test_dir}/links/broken"
        ln -s /proc/1/root "#{test_dir}/links/forbidden"
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
    config.vm.provision :shell, privileged: false, inline: <<-EOF
        set -xe
        mkdir "#{test_dir}/permissions"

        touch     "#{test_dir}/permissions/all-permissions"
        chmod 777 "#{test_dir}/permissions/all-permissions"

        touch     "#{test_dir}/permissions/no-permissions"
        chmod 000 "#{test_dir}/permissions/no-permissions"

        mkdir     "#{test_dir}/permissions/forbidden-directory"
        chmod 000 "#{test_dir}/permissions/forbidden-directory"

        for perms in 001 002 004 010 020 040 100 200 400; do
            touch        "#{test_dir}/permissions/$perms"
            chmod $perms "#{test_dir}/permissions/$perms"
        done

        touch -t #{some_date}      "#{test_dir}/permissions/"*
        sudo chown #{user}:#{user} "#{test_dir}/permissions/"*
    EOF


    # Awkward extended attribute testcases.
    config.vm.provision :shell, privileged: false, inline: <<-EOF
        set -xe
        mkdir "#{test_dir}/attributes"

        touch "#{test_dir}/attributes/none"

        touch "#{test_dir}/attributes/one"
        setfattr -n user.greeting -v hello "#{test_dir}/attributes/one"

        touch "#{test_dir}/attributes/two"
        setfattr -n user.greeting -v hello "#{test_dir}/attributes/two"
        setfattr -n user.another_greeting -v hi "#{test_dir}/attributes/two"

        #touch "#{test_dir}/attributes/forbidden"
        #setfattr -n user.greeting -v hello "#{test_dir}/attributes/forbidden"
        #chmod +a "$YOU deny readextattr" "#{test_dir}/attributes/forbidden"

        mkdir "#{test_dir}/attributes/dirs"

        mkdir "#{test_dir}/attributes/dirs/empty-with-attribute"
        setfattr -n user.greeting -v hello "#{test_dir}/attributes/dirs/empty-with-attribute"

        mkdir "#{test_dir}/attributes/dirs/full-with-attribute"
        touch "#{test_dir}/attributes/dirs/full-with-attribute/file"
        setfattr -n user.greeting -v hello "#{test_dir}/attributes/dirs/full-with-attribute"

        mkdir "#{test_dir}/attributes/dirs/full-but-forbidden"
        touch "#{test_dir}/attributes/dirs/full-but-forbidden/file"
        #setfattr -n user.greeting -v hello "#{test_dir}/attributes/dirs/full-but-forbidden"
        #chmod 000 "#{test_dir}/attributes/dirs/full-but-forbidden"
        #chmod +a "$YOU deny readextattr" "#{test_dir}/attributes/dirs/full-but-forbidden"

        touch -t #{some_date} "#{test_dir}/attributes"
        touch -t #{some_date} "#{test_dir}/attributes/"*
        touch -t #{some_date} "#{test_dir}/attributes/dirs/"*
        touch -t #{some_date} "#{test_dir}/attributes/dirs/"*/*

        sudo chown #{user}:#{user} -R "#{test_dir}/attributes"
    EOF
end
