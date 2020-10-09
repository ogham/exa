all: build test
all-release: build-release test-release


# compiles the exa binary
@build:
    cargo build

# compiles the exa binary (in release mode)
@build-release:
    cargo build --release --verbose


# runs unit tests
@test:
    cargo test --all -- --quiet

# runs unit tests (in release mode)
@test-release:
    cargo test --release --all --verbose


# prints versions of the necessary build tools
@versions:
    rustc --version
    cargo --version
