ARG BASE_RUST=rust:1.71.1

FROM ${BASE_RUST} AS just
# Install Just, the command runner.
RUN curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /usr/bin

FROM ${BASE_RUST} AS specsheet
RUN cargo install -q --git https://github.com/ogham/specsheet

FROM ${BASE_RUST} AS cargo-hack
RUN cargo install -q cargo-hack

FROM ${BASE_RUST} AS cargo-kcov
RUN cargo install -q cargo-kcov

FROM ${BASE_RUST} AS exa
WORKDIR /app
# Some juggling to cache dependencies
RUN <<EOF
    mkdir src
    echo 'fn main() { panic!("Dummy Image Called!")}' > src/main.rs
EOF
COPY Cargo.toml Cargo.lock  ./
RUN cargo build
COPY . .
#need to break the cargo cache
RUN touch ./src/main.rs
# Build exa
RUN cargo build

# We use Ubuntu instead of Debian because the image comes with two-way
# shared folder support by default.
# This image is based from Ubuntu
FROM ${BASE_RUST} AS base

# Install the dependencies needed for exa to build
RUN --mount=type=cache,target=/var/cache/apt apt-get update -qq
RUN --mount=type=cache,target=/var/cache/apt apt-get install -qq -o=Dpkg::Use-Pty=0 \
    git gcc curl attr libgit2-dev zip \
    fish zsh bash bash-completion

# Install kcov for test coverage
# This doesn’t run coverage over the xtests so it’s less useful for now
COPY --from=cargo-kcov /usr/local/cargo/bin/cargo-kcov /usr/bin/cargo-kcov

RUN --mount=type=cache,target=/var/cache/apt apt-get install -y \
    cmake g++ pkg-config \
    libcurl4-openssl-dev libdw-dev binutils-dev libiberty-dev

RUN ln -s $(which python3) /usr/bin/python
RUN cargo kcov --print-install-kcov-sh | sh

# Create a variety of misc scripts.
RUN <<EOF
  ln -sf /vagrant/devtools/dev-run-debug.sh /usr/bin/exa
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
EOF

# Configure the welcoming text that gets shown:
RUN <<EOF
  # Capture the help text so it gets displayed first
  rm -f /etc/update-motd.d/*
  bash /vagrant/devtools/dev-help.sh > /etc/motd

  # Tell bash to execute a bunch of stuff when a session starts
  echo "source /vagrant/devtools/dev-bash.sh" > ${HOME}/.bash_profile

  # Link the completion files so they’re “installed”:

  # bash
  ln -s /vagrant/completions/bash/exa /etc/bash_completion.d/exa
  ln -s /vagrant/completions/bash/exa /usr/share/bash-completion/completions/exa

  # zsh
  ln -s /vagrant/completions/zsh/_exa /usr/share/zsh/vendor-completions/_exa

  # fish
  ln -s /vagrant/completions/fish/exa.fish /usr/share/fish/completions/exa.fish
EOF


# Copy the source code into the image
COPY --chmod=+x . /vagrant/

# Make sudo dummy replacement
# This is needed for some tests that use sudo
RUN echo -e '#!/bin/sh\n"$@"' > /usr/bin/sudo
RUN chmod +x /usr/bin/sudo

RUN bash /vagrant/devtools/dev-set-up-environment.sh
RUN bash /vagrant/devtools/dev-create-test-filesystem.sh

WORKDIR /vagrant
COPY --link --from=exa /app/target /vagrant/target

# TODO: remove this once tests don't depend on it
RUN ln -s /vagrant/* ${HOME}

COPY --link --from=specsheet /usr/local/cargo/bin/specsheet /usr/bin/specsheet
COPY --link --from=cargo-hack /usr/local/cargo/bin/cargo-hack /usr/bin/cargo-hack
COPY --link --from=just /usr/bin/just /usr/bin/just

FROM base AS test
CMD ["/vagrant/xtests/run.sh"]
