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

INSTALL = $(PREFIX)/bin/exa

$(INSTALL):
	# BSD and OSX don't have -D to create leading directories
	install -dm755 -- "$(PREFIX)/bin/" "$(DESTDIR)$(PREFIX)/share/man/man1/"
	install -sm755 -- target/release/exa "$(DESTDIR)$(PREFIX)/bin/"
	install -m644  -- contrib/man/*.1 "$(DESTDIR)$(PREFIX)/share/man/man1/"

install: build $(INSTALL)

.PHONY: install
