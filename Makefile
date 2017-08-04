DESTDIR =
PREFIX  = /usr/local

override define compdir
ifndef $(1)
$(1) := $$(or $$(shell pkg-config --variable=completionsdir $(2) 2>/dev/null),$(3))
endif
endef

$(eval $(call compdir,BASHDIR,bash-completion,$(PREFIX)/etc/bash_completion.d))
ZSHDIR  = /usr/share/zsh/site-functions
$(eval $(call compdir,FISHDIR,fish,$(PREFIX)/share/fish/vendor_completions.d))

FEATURES ?= default


all: target/release/exa
build: target/release/exa

target/release/exa:
	cargo build --release --no-default-features --features "$(FEATURES)"

install: install-exa install-man

install-exa: target/release/exa
	install -m755 -- target/release/exa "$(DESTDIR)$(PREFIX)/bin/"

install-man:
	install -dm755 -- "$(DESTDIR)$(PREFIX)/bin/" "$(DESTDIR)$(PREFIX)/share/man/man1/"
	install -m644  -- contrib/man/exa.1 "$(DESTDIR)$(PREFIX)/share/man/man1/"

install-bash-completions:
	install -m644 -- contrib/completions.bash "$(DESTDIR)$(BASHDIR)/exa"

install-zsh-completions:
	install -m644 -- contrib/completions.zsh "$(DESTDIR)$(ZSHDIR)/_exa"

install-fish-completions:
	install -m644 -- contrib/completions.fish "$(DESTDIR)$(FISHDIR)/exa.fish"

uninstall:
	-rm -f -- "$(DESTDIR)$(PREFIX)/share/man/man1/exa.1"
	-rm -f -- "$(DESTDIR)$(PREFIX)/bin/exa"
	-rm -f -- "$(DESTDIR)$(BASHDIR)/exa"
	-rm -f -- "$(DESTDIR)$(ZSHDIR)/_exa"
	-rm -f -- "$(DESTDIR)$(FISHDIR)/exa.fish"

clean:
	cargo clean

preview-man:
	man contrib/man/exa.1

help:
	@echo 'Available make targets:'
	@echo '  all         - build exa (default)'
	@echo '  build       - build exa'
	@echo '  clean       - run `cargo clean`'
	@echo '  install     - build and install exa and manpage'
	@echo '  install-exa - build and install exa'
	@echo '  install-man - install the manpage'
	@echo '  uninstall   - uninstall fish, manpage, and completions'
	@echo '  preview-man - preview the manpage without installing'
	@echo '  help        - print this help'
	@echo
	@echo '  install-bash-completions - install bash completions into $$BASHDIR'
	@echo '  install-zsh-completions  - install zsh completions into $$ZSHDIR'
	@echo '  install-fish-completions - install fish completions into $$FISHDIR'
	@echo
	@echo 'Variables:'
	@echo '  DESTDIR  - A path that'\''s prepended to installation paths (default: "")'
	@echo '  PREFIX   - The installation prefix for everything except zsh completions (default: /usr/local)'
	@echo '  BASHDIR  - The directory to install bash completions in (default: $$PREFIX/etc/bash_completion.d)'
	@echo '  ZSHDIR   - The directory to install zsh completions in (default: /usr/share/zsh/vendor-completions)'
	@echo '  FISHDIR  - The directory to install fish completions in (default: $$PREFIX/share/fish/vendor_completions.d)'
	@echo '  FEATURES - The cargo feature flags to use. Set to an empty string to disable git support'

.PHONY: all build target/release/exa install-exa install-man preview-man \
	install-bash-completions install-zsh-completions install-fish-completions \
	clean uninstall help
