PREFIX ?= /usr/local

BUILD = target/release/exa

$(BUILD):
	@which rustc > /dev/null || { echo "exa requires Rust to compile. For installation instructions, please visit http://rust-lang.org/"; exit 1; }
	cargo build --release

build: $(BUILD)

build-no-git:
	@which rustc > /dev/null || { echo "exa requires Rust to compile. For installation instructions, please visit http://rust-lang.org/"; exit 1; }
	cargo build --release --no-default-features

INSTALL = $(PREFIX)/bin/exa

$(INSTALL):
	# BSD and OSX don't have -D to create leading directories
	install -dm755 -- "$(PREFIX)/bin/" "$(DESTDIR)$(PREFIX)/share/man/man1/"
	install -sm755 -- target/release/exa "$(DESTDIR)$(PREFIX)/bin/"
	install -m644  -- contrib/man/*.1 "$(DESTDIR)$(PREFIX)/share/man/man1/"

install: build $(INSTALL)

.PHONY: install
