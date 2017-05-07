DESTDIR =
PREFIX  = /usr/local

BASHDIR = /etc/bash_completion.d
ZSHDIR  = /usr/share/zsh/vendor-completions
FISHDIR = /usr/share/fish/completions

FEATURES ?= default


all: target/release/exa
build: target/release/exa

target/release/exa:
	cargo build --release --features "${ENABLE_FEATURES}"

install: install-exa install-man

install-exa: target/release/exa
	install -m755 -- target/release/exa "$(DESTDIR)$(PREFIX)/bin/"

install-man:
	install -dm755 -- "$(DESTDIR)$(PREFIX)/bin/" "$(DESTDIR)$(PREFIX)/share/man/man1/"
	install -m644  -- contrib/man/exa.1 "$(DESTDIR)$(PREFIX)/share/man/man1/"

install-bash-completions:
	install -m644 -- contrib/completions.bash "$(BASHDIR)/exa"

install-zsh-completions:
	install -m644 -- contrib/completions.zsh "$(ZSHDIR)/_exa"

install-fish-completions:
	install -m644 -- contrib/completions.fish "$(FISHDIR)/exa.fish"


uninstall:
	-rm -- "$(DESTDIR)$(PREFIX)/share/man/man1/exa.1"
	-rm -- "$(DESTDIR)$(PREFIX)/bin/exa"
	-rm -- "$(BASHDIR)/exa"
	-rm -- "$(ZSHDIR)/_exa"
	-rm -- "$(FISHDIR)/exa.fish"

clean:
	cargo clean


preview-man:
	nroff -man contrib/man/exa.1 | less


.PHONY: all build install-exa install-man preview-man \
	install-bash-completions install-zsh-completions install-fish-completions \
	clean uninstall
