PREFIX ?= /usr/local

BUILD = target/release/exa

$(BUILD):
	@which rustc > /dev/null || { echo "exa requires Rust Nightly to compile. For installation instructions, please visit http://rust-lang.org/"; exit 1; }
	cargo build --release

build: $(BUILD)

build-no-git:
	@which rustc > /dev/null || { echo "exa requires Rust Nightly to compile. For installation instructions, please visit http://rust-lang.org/"; exit 1; }
	cargo build --release --no-default-features

INSTALL = $(PREFIX)/bin/exa

$(INSTALL):
	cp target/release/exa $(PREFIX)/bin/
	cp contrib/man/*.1 $(PREFIX)/share/man/man1/

install: build $(INSTALL)

.PHONY: install
