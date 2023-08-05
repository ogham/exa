# We use Ubuntu instead of Debian because the image comes with two-way
# shared folder support by default.
FROM ubuntu:22.04 AS base

# Install the dependencies needed for exa to build
RUN apt-get update -qq
RUN apt-get install -qq -o=Dpkg::Use-Pty=0 \
    git gcc curl attr libgit2-dev zip \
    fish zsh bash bash-completion

# Install Rust (cargo + rustup) and the Rust tools we need.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --component rustc,rust-std,cargo,clippy -y

# Add Rust binaries to path
ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo install -q cargo-hack
RUN cargo install -q --git https://github.com/ogham/specsheet

# Install Just, the command runner.
RUN curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /usr/local/bin

# TODO: Guarantee that the timezone is UTC — some of the tests depend on this (for now).
# RUN timedatectl set-timezone UTC

ARG DEVELOPER_HOME=/root
# Use a different ‘target’ directory on the VM than on the host.
# By default it just uses the one in /vagrant/target, which can
# cause problems if it has different permissions than the other
# directories, or contains object files compiled for the host.
RUN echo 'PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:${DEVELOPER_HOME}/.cargo/bin"' > /etc/environment
RUN echo 'CARGO_TARGET_DIR="${DEVELOPER_HOME}/target"' >> /etc/environment

# Create a variety of misc scripts.

RUN ln -sf /vagrant/devtools/dev-run-debug.sh   /usr/bin/exa
RUN ln -sf /vagrant/devtools/dev-run-release.sh /usr/bin/rexa

RUN echo -e "#!/bin/sh\ncargo build --manifest-path /vagrant/Cargo.toml \\$@" > /usr/bin/build-exa
RUN ln -sf /usr/bin/build-exa /usr/bin/b

RUN echo -e "#!/bin/sh\ncargo test --manifest-path /vagrant/Cargo.toml \\$@ -- --quiet" > /usr/bin/test-exa
RUN ln -sf /usr/bin/test-exa /usr/bin/t

RUN echo -e "#!/bin/sh\n/vagrant/xtests/run.sh" > /usr/bin/run-xtests
RUN ln -sf /usr/bin/run-xtests /usr/bin/x

RUN echo -e "#!/bin/sh\nbuild-exa && test-exa && run-xtests" > /usr/bin/compile-exa
RUN ln -sf /usr/bin/compile-exa /usr/bin/c

ADD --chmod=+x devtools/dev-package-for-linux.sh /vagrant/devtools/dev-package-for-linux.sh
RUN echo -e "#!/bin/sh\nbash /vagrant/devtools/dev-package-for-linux.sh \\$@" > /usr/bin/package-exa
RUN echo -e "#!/bin/sh\ncat /etc/motd" > /usr/bin/halp

# RUN chmod +x /usr/bin/{exa,rexa,b,t,x,c,build-exa,test-exa,run-xtests,compile-exa,package-exa,halp}


# Configure the welcoming text that gets shown:

# Capture the help text so it gets displayed first
RUN rm -f /etc/update-motd.d/*
ADD --chmod=+x devtools/dev-help.sh /vagrant/devtools/dev-help.sh
RUN bash /vagrant/devtools/dev-help.sh > /etc/motd

# Tell bash to execute a bunch of stuff when a session starts
RUN echo "source /vagrant/devtools/dev-bash.sh" > ${DEVELOPER_HOME}/.bash_profile

# Disable last login date in sshd
# RUN sed -i '/PrintLastLog yes/c\PrintLastLog no' /etc/ssh/sshd_config
# RUN systemctl restart sshd


# Link the completion files so they’re “installed”:

# bash
RUN test -h /etc/bash_completion.d/exa \
    || ln -s /vagrant/contrib/completions.bash /etc/bash_completion.d/exa

# zsh
RUN test -h /usr/share/zsh/vendor-completions/_exa \
    || ln -s /vagrant/contrib/completions.zsh /usr/share/zsh/vendor-completions/_exa

# fish
RUN test -h /usr/share/fish/completions/exa.fish \
    || ln -s /vagrant/contrib/completions.fish /usr/share/fish/completions/exa.fish

# Install kcov for test coverage
# This doesn’t run coverage over the xtests so it’s less useful for now

RUN test -e ~/.cargo/bin/cargo-kcov \
    || cargo install cargo-kcov

RUN apt-get install -y \
    cmake g++ pkg-config \
    libcurl4-openssl-dev libdw-dev binutils-dev libiberty-dev

RUN ln -s $(which python3) /usr/bin/python
RUN cargo kcov --print-install-kcov-sh | sh

COPY --chmod=+x . /vagrant/

# Make sudo dummy replacement, so we don't weaken docker security
RUN echo -e "#!/bin/bash\n\$@" > /usr/bin/sudo
RUN chmod +x /usr/bin/sudo

RUN bash /vagrant/devtools/dev-set-up-environment.sh
RUN bash /vagrant/devtools/dev-create-test-filesystem.sh

WORKDIR /vagrant
RUN cargo build
RUN bash /usr/bin/build-exa

FROM base AS test
CMD ["/vagrant/xtests/run.sh"]
