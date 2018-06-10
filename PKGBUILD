# Maintainer: David Flemström <david.flemstrom@gmail.com>
pkgname=precip
pkgver=0.1.0
pkgrel=1
pkgdesc='irrigation control system'
arch=('x86_64', 'i686', 'arm', 'armv6h', 'armv7h', 'aarch64')
url='https://github.com/dflemstr/precip'
license=('MIT')
depends=('libsystemd' 'postgresql-libs')
makedepends=('rustup' 'git')
source=('git+https://github.com/dflemstr/precip.git') # TODO: use version number
md5sums=('SKIP')

build() {
  cd "$pkgname"
  rustup toolchain install $(cat rust-toolchain)
  cargo +$(cat rust-toolchain) build --release
}

package() {
  cd "$pkgname"
  mkdir -p "$pkgdir/etc/precip"
  mkdir -p "$pkgdir/etc/systemd"
  mkdir -p "$pkgdir/usr/bin"

  install -Dm644 precip.service "$pkgdir/etc/systemd/system"
  install -Dm644 config.toml "$pkgdir/etc/precip"
  install -Dm755 target/release/precip "$pkgdir/usr/bin"
}
