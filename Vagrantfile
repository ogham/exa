Vagrant.configure(2) do |config|

  # We use Ubuntu instead of Debian because the image comes with two-way
  # shared folder support by default.
  UBUNTU = 'hashicorp/bionic64'

  config.vm.define(:exa) do |config|
    config.vm.provider :virtualbox do |v|
      v.name = 'exa'
      v.memory = 2048
      v.cpus = `nproc`.chomp.to_i
    end

    config.vm.provider :vmware_desktop do |v|
      v.vmx['memsize'] = '2048'
      v.vmx['numvcpus'] = `nproc`.chomp
    end

    config.vm.box = UBUNTU
    config.vm.hostname = 'exa'


    # Make sure we know the VM image’s default user name. The ‘cassowary’ user
    # (specified later) is used for most of the test *output*, but we still
    # need to know where the ‘target’ and ‘.cargo’ directories go.
    developer = 'vagrant'


    # Install the dependencies needed for exa to build, as quietly as
    # apt can do.
    config.vm.provision :shell, privileged: true, inline: <<-EOF
      if hash fish &>/dev/null; then
        echo "Tools are already installed"
      else
        trap 'exit' ERR
        echo "Installing tools"
        apt-get update -qq
        apt-get install -qq -o=Dpkg::Use-Pty=0 \
          git gcc curl attr libgit2-dev zip \
          fish zsh bash bash-completion
      fi
    EOF


    # Install Rust.
    # This is done as vagrant, not root, because it’s vagrant
    # who actually uses it. Sent to /dev/null because the progress
    # bar produces a ton of output.
    config.vm.provision :shell, privileged: false, inline: <<-EOF
      if hash rustc &>/dev/null; then
        echo "Rust is already installed"
      else
        trap 'exit' ERR
        echo "Installing Rust"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal --component rustc,rust-std,cargo,clippy -y
        source $HOME/.cargo/env
        cargo install -q cargo-hack
      fi
    EOF


    # Privileged installation and setup scripts.
    config.vm.provision :shell, privileged: true, inline: <<-EOF

      # Install Just, the command runner.
      if hash just &>/dev/null; then
        echo "just is already installed"
      else
        trap 'exit' ERR
        echo "Installing just"
        wget -q "https://github.com/casey/just/releases/download/v0.8.0/just-v0.8.0-x86_64-unknown-linux-musl.tar.gz"
        tar -xf "just-v0.8.0-x86_64-unknown-linux-musl.tar.gz"
        cp just /usr/local/bin
      fi


      # Guarantee that the timezone is UTC — some of the tests
      # depend on this (for now).
      timedatectl set-timezone UTC


      # Use a different ‘target’ directory on the VM than on the host.
      # By default it just uses the one in /vagrant/target, which can
      # cause problems if it has different permissions than the other
      # directories, or contains object files compiled for the host.
      echo 'PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/home/#{developer}/.cargo/bin"' > /etc/environment
      echo 'CARGO_TARGET_DIR="/home/#{developer}/target"'                                                     >> /etc/environment


      # Create a variety of misc scripts.

      ln -sf /vagrant/devtools/dev-run-debug.sh   /usr/bin/exa
      ln -sf /vagrant/devtools/dev-run-release.sh /usr/bin/rexa

      echo -e "#!/bin/sh\ncargo build --manifest-path /vagrant/Cargo.toml \\$@" > /usr/bin/build-exa
      ln -sf /usr/bin/build-exa /usr/bin/b

      echo -e "#!/bin/sh\ncargo test --manifest-path /vagrant/Cargo.toml \\$@ -- --quiet" > /usr/bin/test-exa
      ln -sf /usr/bin/test-exa /usr/bin/t

      echo -e "#!/bin/sh\n/vagrant/xtests/run.sh" > /usr/bin/run-xtests
      ln -sf /usr/bin/run-xtests /usr/bin/x

      echo -e "#!/bin/sh\nbuild-exa && test-exa && run-xtests" > /usr/bin/compile-exa
      ln -sf /usr/bin/compile-exa /usr/bin/c

      echo -e "#!/bin/sh\nbash /vagrant/devtools/dev-package-for-linux.sh \\$@" > /usr/bin/package-exa
      echo -e "#!/bin/sh\ncat /etc/motd" > /usr/bin/halp

      chmod +x /usr/bin/{exa,rexa,b,t,x,c,build-exa,test-exa,run-xtests,compile-exa,package-exa,halp}


      # Configure the welcoming text that gets shown:

      # Capture the help text so it gets displayed first
      rm -f /etc/update-motd.d/*
      bash /vagrant/devtools/dev-help.sh > /etc/motd

      # Tell bash to execute a bunch of stuff when a session starts
      echo "source /vagrant/devtools/dev-bash.sh" > /home/#{developer}/.bash_profile
      chown #{developer} /home/#{developer}/.bash_profile

      # Disable last login date in sshd
      sed -i '/PrintLastLog yes/c\PrintLastLog no' /etc/ssh/sshd_config
      systemctl restart sshd


      # Link the completion files so they’re “installed”:

      # bash
      test -h /etc/bash_completion.d/exa \
        || ln -s /vagrant/contrib/completions.bash /etc/bash_completion.d/exa

      # zsh
      test -h /usr/share/zsh/vendor-completions/_exa \
        || ln -s /vagrant/contrib/completions.zsh /usr/share/zsh/vendor-completions/_exa

      # fish
      test -h /usr/share/fish/completions/exa.fish \
        || ln -s /vagrant/contrib/completions.fish /usr/share/fish/completions/exa.fish
    EOF


    # Install kcov for test coverage
    # This doesn’t run coverage over the xtests so it’s less useful for now
    if ENV.key?('INSTALL_KCOV')
      config.vm.provision :shell, privileged: false, inline: <<-EOF
        trap 'exit' ERR

        test -e ~/.cargo/bin/cargo-kcov \
          || cargo install cargo-kcov

        sudo apt-get install -qq -o=Dpkg::Use-Pty=0 -y \
          cmake g++ pkg-config \
          libcurl4-openssl-dev libdw-dev binutils-dev libiberty-dev

        cargo kcov --print-install-kcov-sh | sudo sh
      EOF
    end

    config.vm.provision :shell, privileged: true,  path: 'devtools/dev-set-up-environment.sh'
    config.vm.provision :shell, privileged: false, path: 'devtools/dev-create-test-filesystem.sh'
  end
end
