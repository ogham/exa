Vagrant.configure("2") do |config|
    config.vm.provider "virtualbox" do |v|
        v.memory = 1024
        v.cpus = 1
    end

    config.vm.box = "debian/contrib-jessie64"
    config.vm.hostname = "exa"

    # Install the dependencies needed for exa to build.
    config.vm.provision :shell, privileged: true, inline:
        %[apt-get install -y git cmake libgit2-dev libssh2-1-dev curl attr]

    # Guarantee that the timezone is UTC -- some of the tests
    # depend on this (for now).
    config.vm.provision :shell, privileged: true, inline:
        %[timedatectl set-timezone UTC]

    # Install Rust.
    # This is done as vagrant, not root, because it’s vagrant
    # who actually uses it. Sent to /dev/null because the progress
    # bar produces a lot of output.
    config.vm.provision :shell, privileged: false, inline:
        %[hash rustc &>/dev/null || curl -sSf https://static.rust-lang.org/rustup.sh | sh &> /dev/null]

    # Use a different ‘target’ directory on the VM than on the host.
    # By default it just uses the one in /vagrant/target, which can
    # cause problems if it has different permissions than the other
    # directories, or contains object files compiled for the host.
    config.vm.provision :shell, privileged: false, inline:
        %[echo "export CARGO_TARGET_DIR=/home/vagrant/target" >> ~/.bashrc]

    # Test that wide columns work with a really long username.
    # The benefit of Vagrant is that we don’t need to set this up
    # on the *actual* system!
    longuser = "antidisestablishmentarienism"
    config.vm.provision :shell, privileged: true, inline:
        %[id -u #{longuser} &>/dev/null || useradd #{longuser}]

    test_dir = "/home/vagrant/testcases"
    invalid_uid = 666
    invalid_gid = 616
    some_date = "201601011234.56"  # 1st January 2016, 12:34:56

    # Delete old testcases if they exist already.
    # This needs root because the generator does some sudo-ing.
    config.vm.provision :shell, privileged: true, inline:
        %[rm -rfv #{test_dir}]

    # Generate our awkward testcases.
    config.vm.provision :shell, privileged: false, inline:
        %[mkdir #{test_dir}]

    # Awkward file size testcases.
    config.vm.provision :shell, privileged: false, inline: <<-EOF
        set -xe
        mkdir "#{test_dir}/files"
        for i in {1..13}; do
            fallocate -l "$i" "#{test_dir}/files/$i"_bytes
            fallocate -l "$i"KiB "#{test_dir}/files/$i"_KiB
            fallocate -l "$i"MiB "#{test_dir}/files/$i"_MiB
        done
        touch -t #{some_date} "#{test_dir}/files/"*
    EOF

    # File name extension testcases.
    config.vm.provision :shell, privileged: false, inline: <<-EOF
        set -xe
        mkdir "#{test_dir}/file-types"

        touch "#{test_dir}/file-types/Makefile"

        touch "#{test_dir}/file-types/image.png"
        touch "#{test_dir}/file-types/image.svg"

        touch "#{test_dir}/file-types/video.avi"
        touch "#{test_dir}/file-types/video.wmv"

        touch "#{test_dir}/file-types/music.mp3"
        touch "#{test_dir}/file-types/music.ogg"

        touch "#{test_dir}/file-types/lossless.flac"
        touch "#{test_dir}/file-types/lossless.wav"

        touch "#{test_dir}/file-types/crypto.asc"
        touch "#{test_dir}/file-types/crypto.signature"

        touch "#{test_dir}/file-types/document.pdf"
        touch "#{test_dir}/file-types/document.xlsx"

        touch "#{test_dir}/file-types/compressed.zip"
        touch "#{test_dir}/file-types/compressed.tar.gz"
        touch "#{test_dir}/file-types/compressed.tgz"

        touch "#{test_dir}/file-types/backup~"
        touch "#{test_dir}/file-types/#SAVEFILE#"
        touch "#{test_dir}/file-types/file.tmp"

        touch "#{test_dir}/file-types/compiled.class"
        touch "#{test_dir}/file-types/compiled.o"
        touch "#{test_dir}/file-types/compiled.js"
        touch "#{test_dir}/file-types/compiled.coffee"

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

        touch -t #{some_date} "#{test_dir}/passwd/unknown-uid"
        sudo chown #{invalid_uid} "#{test_dir}/passwd/unknown-uid"

        touch -t #{some_date} "#{test_dir}/passwd/unknown-gid"
        sudo chgrp #{invalid_gid} "#{test_dir}/passwd/unknown-gid"
    EOF

    # Awkward permission testcases.
    config.vm.provision :shell, privileged: false, inline: <<-EOF
        set -xe
        mkdir "#{test_dir}/permissions"

        touch "#{test_dir}/permissions/all-permissions"
        chmod 777 "#{test_dir}/permissions/all-permissions"

        touch "#{test_dir}/permissions/no-permissions"
        chmod 000 "#{test_dir}/permissions/no-permissions"

        mkdir "#{test_dir}/permissions/forbidden-directory"
        chmod 000 "#{test_dir}/permissions/forbidden-directory"

        touch -t #{some_date} "#{test_dir}/permissions/"*
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
    EOF
end
