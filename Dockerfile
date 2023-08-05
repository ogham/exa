FROM rust:1.71.1 AS just
# Install Just, the command runner.
RUN curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /usr/bin

FROM rust:1.71.1 AS specsheet
RUN cargo install -q --git https://github.com/ogham/specsheet

FROM rust:1.71.1 AS cargo-hack
RUN cargo install -q cargo-hack

FROM rust:1.71.1 AS exa
WORKDIR /app
# Copy the source code into the image
COPY . .
# Build exa
RUN cargo build

# We use Ubuntu instead of Debian because the image comes with two-way
# shared folder support by default.
# This image is based from Ubuntu
FROM rust:1.71.1 AS base

# Install the dependencies needed for exa to build
RUN apt-get update -qq
RUN apt-get install -qq -o=Dpkg::Use-Pty=0 \
    git gcc curl attr libgit2-dev zip \
    fish zsh bash bash-completion

# TODO: Guarantee that the timezone is UTC — some of the tests depend on this (for now).
# RUN timedatectl set-timezone UTC

# Create a variety of misc scripts.

RUN ln -sf /vagrant/devtools/dev-run-debug.sh /usr/bin/exa
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
RUN echo "source /vagrant/devtools/dev-bash.sh" > ${HOME}/.bash_profile

# Disable last login date in sshd
# RUN sed -i '/PrintLastLog yes/c\PrintLastLog no' /etc/ssh/sshd_config
# RUN systemctl restart sshd


# Link the completion files so they’re “installed”:

# bash
RUN ln -s /vagrant/completions/bash/exa /etc/bash_completion.d/exa
RUN ln -s /vagrant/completions/bash/exa /usr/share/bash-completion/completions/exa

# zsh
RUN ln -s /vagrant/completions/zsh/_exa /usr/share/zsh/vendor-completions/_exa

# fish
RUN ln -s /vagrant/completions/fish/exa.fish /usr/share/fish/completions/exa.fish

# Install kcov for test coverage
# This doesn’t run coverage over the xtests so it’s less useful for now

RUN cargo install cargo-kcov

RUN apt-get install -y \
    cmake g++ pkg-config \
    libcurl4-openssl-dev libdw-dev binutils-dev libiberty-dev

RUN ln -s $(which python3) /usr/bin/python
RUN cargo kcov --print-install-kcov-sh | sh

# Copy the source code into the image
COPY --chmod=+x . /vagrant/

# Make sudo dummy replacement, so we don't weaken docker security
RUN echo -e "#!/bin/bash\n\$@" > /usr/bin/sudo
RUN chmod +x /usr/bin/sudo

RUN bash /vagrant/devtools/dev-set-up-environment.sh
RUN bash /vagrant/devtools/dev-create-test-filesystem.sh

WORKDIR /vagrant
COPY --from=exa /app/target /vagrant/target

# TODO: remove this once tests don't depend on it
RUN ln -s /vagrant/* ${HOME}

COPY --from=specsheet /usr/local/cargo/bin/specsheet /usr/bin/specsheet
COPY --from=cargo-hack /usr/local/cargo/bin/cargo-hack /usr/bin/cargo-hack
COPY --from=just /usr/bin/just /usr/bin/just

FROM base AS test
CMD ["/vagrant/xtests/run.sh"]
