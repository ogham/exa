PREFIX = /usr/local

CARGOFLAGS = --no-default-features

all: target/release/exa

build: CARGOFLAGS=
build: all
build-no-git: all

target/release/exa:
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

.PHONY: all build build-no-git install uninstall clean
