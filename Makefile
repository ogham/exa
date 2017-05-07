SRC = \
	src/info/sources.rs \
	src/info/mod.rs \
	src/info/filetype.rs \
	src/bin/main.rs \
	src/term.rs \
	src/exa.rs \
	src/output/grid_details.rs \
	src/output/tree.rs \
	src/output/colours.rs \
	src/output/grid.rs \
	src/output/cell.rs \
	src/output/mod.rs \
	src/output/details.rs \
	src/output/lines.rs \
	src/output/column.rs \
	src/fs/file.rs \
	src/fs/fields.rs \
	src/fs/mod.rs \
	src/fs/dir.rs \
	src/fs/feature/xattr.rs \
	src/fs/feature/git.rs \
	src/fs/feature/mod.rs \
	src/options/misfire.rs \
	src/options/filter.rs \
	src/options/dir_action.rs \
	src/options/view.rs \
	src/options/mod.rs \
	src/options/help.rs

PREFIX = /usr/local

CARGOFLAGS = --no-default-features

all: target/release/exa

build: CARGOFLAGS=
build: all
build-no-git: all

target/release/exa: $(SRC)
	if test -n "$$(echo "$$CC" | cut -d \  -f 1)"; then \
	    env CC="$$(echo "$$CC" | cut -d \  -f 1)" cargo build --release $(CARGOFLAGS); \
	else\
	    env -u CC cargo build --release $(CARGOFLAGS); \
	fi

install: target/release/exa
	# BSD and OSX don't have -D to create leading directories
	install -dm755 -- "$(DESTDIR)$(PREFIX)/bin/" "$(DESTDIR)$(PREFIX)/share/man/man1/"
	install -m755 -- target/release/exa "$(DESTDIR)$(PREFIX)/bin/"
	install -m644  -- contrib/man/exa.1 "$(DESTDIR)$(PREFIX)/share/man/man1/"

uninstall:
	-rm    -- "$(DESTDIR)$(PREFIX)/share/man/man1/exa.1"
	-rmdir -- "$(DESTDIR)$(PREFIX)/share/man/man1"
	-rm    -- "$(DESTDIR)$(PREFIX)/bin/exa"
	-rmdir -- "$(DESTDIR)$(PREFIX)/bin"

clean:
	-rm -rf target

.PHONY: all build install
