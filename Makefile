.PHONY: run install reinstall uninstall clean

BINARY := claude-genmon
PREFIX := $(HOME)/.local/bin
DATA_DIR := $(HOME)/.local/share/claude-genmon

run:
	cargo run

install:
	cargo build --release
	install -Dm755 target/release/$(BINARY) $(PREFIX)/$(BINARY)
	install -Dm644 assets/anthropic.png $(DATA_DIR)/anthropic.png

uninstall:
	-rm -f $(PREFIX)/$(BINARY)
	-rm -f $(DATA_DIR)/anthropic.png

clean: uninstall
	cargo clean
