all: build test
all-release: build-release test-release


#----------#
# building #
#----------#

# compile the exa binary
@build:
    cargo build

# compile the exa binary (in release mode)
@build-release:
    cargo build --release --verbose

# produce an HTML chart of compilation timings
@build-time:
    cargo +nightly clean
    cargo +nightly build -Z timings

# check that the exa binary can compile
@check:
    cargo check


#---------------#
# running tests #
#---------------#

# run unit tests
@test:
    cargo test --workspace -- --quiet

# run unit tests (in release mode)
@test-release:
    cargo test --workspace --release --verbose


#------------------------#
# running extended tests #
#------------------------#

# run extended tests
@xtests:
    xtests/run.sh

# run extended tests (using the release mode exa)
@xtests-release:
    xtests/run.sh --release

# display the number of extended tests that get run
@count-xtests:
    grep -F '[[cmd]]' -R xtests | wc -l


#-----------------------#
# code quality and misc #
#-----------------------#

# lint the code
@clippy:
    touch src/main.rs
    cargo clippy

# update dependency versions, and checks for outdated ones
@update-deps:
    cargo update
    command -v cargo-outdated >/dev/null || (echo "cargo-outdated not installed" && exit 1)
    cargo outdated

# list unused dependencies
@unused-deps:
    command -v cargo-udeps >/dev/null || (echo "cargo-udeps not installed" && exit 1)
    cargo +nightly udeps

# check that every combination of feature flags is successful
@check-features:
    command -v cargo-hack >/dev/null || (echo "cargo-hack not installed" && exit 1)
    cargo hack check --feature-powerset

# build exa and run extended tests with features disabled
@feature-checks *args:
    cargo build --no-default-features
    specsheet xtests/features/none.toml -shide {{args}} \
        -O cmd.target.exa="${CARGO_TARGET_DIR:-../../target}/debug/exa"

# print versions of the necessary build tools
@versions:
    rustc --version
    cargo --version


#---------------#
# documentation #
#---------------#

# build the man pages
@man:
    mkdir -p "${CARGO_TARGET_DIR:-target}/man"
    pandoc --standalone -f markdown -t man man/exa.1.md        > "${CARGO_TARGET_DIR:-target}/man/exa.1"
    pandoc --standalone -f markdown -t man man/exa_colors.5.md > "${CARGO_TARGET_DIR:-target}/man/exa_colors.5"

# build and preview the main man page (exa.1)
@man-1-preview: man
    man "${CARGO_TARGET_DIR:-target}/man/exa.1"

# build and preview the colour configuration man page (exa_colors.5)
@man-5-preview: man
    man "${CARGO_TARGET_DIR:-target}/man/exa_colors.5"
