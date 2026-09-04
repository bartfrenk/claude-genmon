.PHONY: run install reinstall uninstall clean

BINARY := claude-genmon
PREFIX := $(HOME)/.local/bin

run:
	cargo run

install:
	cargo build --release
	install -Dm755 target/release/$(BINARY) $(PREFIX)/$(BINARY)

uninstall:
	-rm -f $(PREFIX)/$(BINARY)

clean: uninstall
	cargo clean
