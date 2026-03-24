PREFIX ?= /usr
DESTDIR ?=

BIN_DIR = $(DESTDIR)$(PREFIX)/bin
APP_DIR = $(DESTDIR)$(PREFIX)/share/applications

build:
	cargo build --release

install: build
	install -Dm755 target/release/omarchy-imageview $(BIN_DIR)/omarchy-imageview
	install -Dm644 omarchy-imageview.desktop $(APP_DIR)/omarchy-imageview.desktop

uninstall:
	rm -f $(BIN_DIR)/omarchy-imageview
	rm -f $(APP_DIR)/omarchy-imageview.desktop

.PHONY: build install uninstall
