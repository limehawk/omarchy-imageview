PREFIX ?= /usr
DESTDIR ?=

INSTALL_DIR = $(DESTDIR)$(PREFIX)/lib/omarchy-imageview
BIN_DIR = $(DESTDIR)$(PREFIX)/bin
APP_DIR = $(DESTDIR)$(PREFIX)/share/applications

install:
	install -d $(INSTALL_DIR)/src/core
	install -d $(INSTALL_DIR)/src/state
	install -d $(INSTALL_DIR)/src/ui
	install -d $(INSTALL_DIR)/src/actions
	install -m 644 src/*.py $(INSTALL_DIR)/src/
	install -m 644 src/core/*.py $(INSTALL_DIR)/src/core/
	install -m 644 src/state/*.py $(INSTALL_DIR)/src/state/
	install -m 644 src/ui/*.py $(INSTALL_DIR)/src/ui/
	install -m 644 src/actions/*.py $(INSTALL_DIR)/src/actions/
	install -d $(BIN_DIR)
	printf '#!/bin/sh\ncd $(PREFIX)/lib/omarchy-imageview && exec python3 -m src.main "$$@"\n' > $(BIN_DIR)/omarchy-imageview
	chmod 755 $(BIN_DIR)/omarchy-imageview
	install -d $(APP_DIR)
	install -m 644 omarchy-imageview.desktop $(APP_DIR)/

uninstall:
	rm -rf $(DESTDIR)$(PREFIX)/lib/omarchy-imageview
	rm -f $(BIN_DIR)/omarchy-imageview
	rm -f $(APP_DIR)/omarchy-imageview.desktop

.PHONY: install uninstall
