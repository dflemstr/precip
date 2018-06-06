pkgname=precip
pkgver=0.8.0
pkgrel=1
pkgdesc='irrigation control system'
arch=('x86_64', 'i686', 'arm', 'armv6h', 'armv7h', 'aarch64')
url='https://github.com/dflemstr/precip'
license=('MIT')
depends=('')
makedepends=('rust', 'cargo', 'git')
source=("git+https://github.com/dflemstr/precip.git#tag=v$pkgver")
md5sums=('SKIP')

build() {
  cargo build --release
}

package() {
  cd "$pkgname"
  install -Dm644 precip.service "$pkgdir/etc/systemd/system"
  install -Dm644 precip config.toml "$pkgdir/etc/precip"
  install -Dm755 target/release/precip "$pkgdir/usr/bin"
}
