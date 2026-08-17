PREFIX ?= /usr
DESTDIR ?=

BIN_DIR = $(DESTDIR)$(PREFIX)/bin
APP_DIR = $(DESTDIR)$(PREFIX)/share/applications

MIME_TYPES = \
	image/jpeg image/png image/webp image/gif image/bmp image/tiff \
	image/avif image/heic image/heif image/jxl image/svg+xml \
	image/x-icon image/vnd.microsoft.icon image/x-tga image/qoi \
	image/vnd.radiance image/vnd-ms.dds \
	image/x-portable-pixmap image/x-portable-graymap \
	image/x-portable-bitmap image/x-portable-anymap \
	image/x-farbfeld image/x-exr

build:
	cargo build --release

install: build
	install -Dm755 target/release/omarchy-imageview $(BIN_DIR)/omarchy-imageview
	install -Dm644 omarchy-imageview.desktop $(APP_DIR)/omarchy-imageview.desktop

# User-local default: make PREFIX=$(HOME)/.local install-as-default
# File managers launched by systemd do not have ~/.local/bin on PATH, so the
# installed desktop file must use an absolute Exec.
install-as-default: install
	sed -i 's|^Exec=omarchy-imageview %F|Exec=$(abspath $(BIN_DIR))/omarchy-imageview %F|' $(APP_DIR)/omarchy-imageview.desktop
	update-desktop-database $(APP_DIR)
	@for t in $(MIME_TYPES); do xdg-mime default omarchy-imageview.desktop $$t; done

uninstall:
	rm -f $(BIN_DIR)/omarchy-imageview
	rm -f $(APP_DIR)/omarchy-imageview.desktop

.PHONY: build install install-as-default uninstall
