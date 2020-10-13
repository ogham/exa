all: build test
all-release: build-release test-release


# compiles the exa binary
@build:
    cargo build

# compiles the exa binary (in release mode)
@build-release:
    cargo build --release --verbose

# compiles the exa binary with every combination of feature flags
@build-features:
    cargo hack build --feature-powerset


# runs unit tests
@test:
    cargo test --all -- --quiet

# runs unit tests (in release mode)
@test-release:
    cargo test --release --all --verbose

# runs unit tests with every combination of feature flags
@test-features:
    cargo hack test --feature-powerset -- --quiet


# lints the code
@clippy:
    touch src/main.rs
    cargo clippy

# updates dependency versions, and checks for outdated ones
@update:
    cargo update
    cargo outdated

# prints versions of the necessary build tools
@versions:
    rustc --version
    cargo --version


# builds the man pages
@man:
    mkdir -p "${CARGO_TARGET_DIR:-target}/man"
    pandoc --standalone -f markdown -t man man/exa.1.md        > "${CARGO_TARGET_DIR:-target}/man/exa.1"
    pandoc --standalone -f markdown -t man man/exa_colors.5.md > "${CARGO_TARGET_DIR:-target}/man/exa_colors.5"

# builds and previews the main man page (exa.1)
@man-1-preview: man
    man "${CARGO_TARGET_DIR:-target}/man/exa.1"

# builds and previews the colour configuration man page (exa_colors.5)
@man-5-preview: man
    man "${CARGO_TARGET_DIR:-target}/man/exa_colors.5"
