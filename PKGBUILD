# Maintainer: Evan Pratten <ewpratten@gmail.com>
#
# This PKGBUILD was generated by `cargo aur`: https://crates.io/crates/cargo-aur

pkgname=syscord-bin
pkgver=0.1.0
pkgrel=1
pkgdesc="Display system status as Discord Rich Presence"
url="https://github.com/ewpratten/syscord"
license=("GPL-3.0")
arch=("x86_64")
provides=("syscord")
conflicts=("syscord")
source=("https://github.com/ewpratten/syscord/releases/download/v$pkgver/syscord-$pkgver-x86_64.tar.gz")
sha256sums=("8f5ed25c0b33f07acc213d7e77766151c71a35eccb73228eca40ed1def135ab8")

package() {
    install -Dm755 syscord -t "$pkgdir/usr/bin"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
