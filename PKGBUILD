pkgname=omarchy-imageview
pkgver=0.1.0
pkgrel=1
pkgdesc="Image viewer and browser for the Omarchy desktop"
arch=('x86_64')
license=('MIT')
depends=('gtk4' 'librsvg')
makedepends=('rust' 'cargo')
source=()

build() {
    cd "$srcdir/.."
    cargo build --release
}

package() {
    cd "$srcdir/.."
    make DESTDIR="$pkgdir" install
}
