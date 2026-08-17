# Maintainer: Limehawk <128890849+limehawk@users.noreply.github.com>
pkgname=omarchy-imageview
pkgver=0.3.1
pkgrel=1
pkgdesc="Image viewer and browser for the Omarchy desktop"
arch=('x86_64')
url="https://github.com/limehawk/omarchy-imageview"
license=('MIT')
depends=('gtk4' 'librsvg' 'libheif')
optdepends=('perl-image-exiftool: lossless JPEG/TIFF rotation')
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
