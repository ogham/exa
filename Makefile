PREFIX ?= /usr/local

BUILD = target/release/exa

$(BUILD):
	if test -n "$$(echo "$$CC" | cut -d \  -f 1)"; then \
	    env CC="$$(echo "$$CC" | cut -d \  -f 1)" cargo build --release; \
	else\
	    env -u CC cargo build --release; \
	fi

build: $(BUILD)

build-no-git:
	if test -n "$$(echo "$$CC" | cut -d \  -f 1)"; then \
	    env CC="$$(echo "$$CC" | cut -d \  -f 1)" cargo build --release --no-default-features; \
	else\
	    env -u CC cargo build --release --no-default-features; \
	fi

install: target/release/exa
	# BSD and OSX don't have -D to create leading directories
	install -dm755 -- "$(PREFIX)/bin/" "$(DESTDIR)$(PREFIX)/share/man/man1/"
	install -sm755 -- target/release/exa "$(DESTDIR)$(PREFIX)/bin/"
	install -m644  -- contrib/man/exa.1 "$(DESTDIR)$(PREFIX)/share/man/man1/"

uninstall:
	-rm    -- "$(DESTDIR)$(PREFIX)/share/man/man1/exa.1"
	-rmdir -- "$(DESTDIR)$(PREFIX)/share/man/man1"
	-rm    -- "$(DESTDIR)$(PREFIX)/bin/exa"
	-rmdir -- "$(DESTDIR)$(PREFIX)/bin"

.PHONY: install uninstall
