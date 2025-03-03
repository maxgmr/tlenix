#!/usr/bin/env bash

KERNEL_VERSION="6.13.5"
KERNEL_MAJOR=$(echo $KERNEL_VERSION | sed 's/\([0-9]*\)[^0-9].*/\1/')
LINUX_NAME=linux-$KERNEL_VERSION
PKG_VER=${KERNEL_VERSION}.arch1
SRC_NAME=linux-${PKG_VER%.*}          # linux-X.Y.Z
SRC_TAG=v${PKG_VER%.*}-${PKG_VER##*.} # vX.Y.Z-arch1

mkdir -p src || exit 1
cd src

# download kernel
wget https://www.kernel.org/pub/linux/kernel/v${KERNEL_MAJOR}.x/${LINUX_NAME}.tar.{xz,sign} || exit 1

# verify kernel integrity
gpg2 --locate-keys torvalds@kernel.org gregkh@kernel.org || exit 1
gpg2 --tofu-policy good 38DBBDC86092693E || exit 1
xz -cd ${LINUX_NAME}.tar.xz | gpg2 --trust-model tofu --verify ${LINUX_NAME}.tar.sign - || exit 1
rm ${LINUX_NAME}.tar.sign

# download arch linux patch
wget https://github.com/archlinux/linux/releases/download/${SRC_TAG}/linux-${SRC_TAG}.patch.zst{,.sig}

# verify patch integrity
wget https://gitlab.archlinux.org/archlinux/packaging/packages/linux/-/raw/main/keys/pgp/83BC8889351B5DEBBB68416EB8AC08600F108CDF.asc || exit 1
gpg2 --import 83BC8889351B5DEBBB68416EB8AC08600F108CDF.asc || exit 1
gpg2 --tofu-policy good 83BC8889351B5DEBBB68416EB8AC08600F108CDF || exit 1
gpg2 --trust-model tofu --verify linux-${SRC_TAG}.patch.zst.sig linux-${SRC_TAG}.patch.zst || exit 1
rm 83BC8889351B5DEBBB68416EB8AC08600F108CDF.asc linux-${SRC_TAG}.patch.zst.sig

# extract kernel
tar -xf ${LINUX_NAME}.tar.xz && rm ${LINUX_NAME}.tar.xz
cd ${LINUX_NAME}

# patch kernel
# patch -Np1 <"../linux-${SRC_TAG}.patch.zst"
