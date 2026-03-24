pkgname=omarchy-imageview
pkgver=0.1.0
pkgrel=1
pkgdesc="Image viewer and browser for the Omarchy desktop"
arch=('any')
license=('MIT')
depends=(
    'python>=3.11'
    'python-gobject'
    'gtk4'
    'python-pillow'
    'python-natsort'
    'librsvg'
    'libheif'
    'libraw'
    'libavif'
)
optdepends=(
    'python-pillow-heif: HEIC/HEIF/AVIF support'
    'python-rawpy: RAW camera format support'
)
source=()

package() {
    cd "$srcdir/.."
    make DESTDIR="$pkgdir" install
}
