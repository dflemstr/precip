# Maintainer: David Flemström <david.flemstrom@gmail.com>
pkgname=precip
pkgver=0.1.0.r151.g06cc112
pkgrel=1
pkgdesc='irrigation control system'
arch=('x86_64' 'armv7h')
url='https://github.com/dflemstr/precip'
license=('MIT')
depends=('libsystemd' 'postgresql-libs')
makedepends=('rustup' 'git')
source=('git+https://github.com/dflemstr/precip.git') # TODO: use version number
md5sums=('SKIP')

pkgver() {
  cd "$pkgname"
  git describe --long --tags | tr -d v | sed 's/\([^-]*-g\)/r\1/;s/-/./g'
}

build() {
  cd "$pkgname"
  rustup toolchain install $(cat rust-toolchain)
  cargo +$(cat rust-toolchain) build --release
}

package() {
  cd "$pkgname"
  mkdir -p "$pkgdir/etc/precip"
  mkdir -p "$pkgdir/usr/lib/systemd/system"
  mkdir -p "$pkgdir/usr/bin"

  install -Dm644 precip.service "$pkgdir/usr/lib/systemd/system"
  install -Dm644 config.toml "$pkgdir/etc/precip"
  install -Dm755 target/release/precip "$pkgdir/usr/bin"
}
