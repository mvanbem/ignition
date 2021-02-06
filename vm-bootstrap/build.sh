#!/bin/bash
set -e

function build_linux {
    rm -rf packages/linux-5.10.13
    rm -rf build/linux
    (cd packages && tar xf linux-5.10.13.tar.xz)
    mkdir -p build/linux
    (cd packages/linux-5.10.13 && make O=../../build/linux allnoconfig)
    cp configs/linux.config build/linux/.config
    (cd build/linux && make -j4)
}

function build_busybox {
    rm -rf packages/busybox-1.33.0
    rm -rf build/busybox
    (cd packages && tar xf busybox-1.33.0.tar.bz2)
    mkdir -p build/busybox
    (cd packages/busybox-1.33.0 && make O=../../build/busybox defconfig)
    cp configs/busybox.config build/busybox/.config
    (cd build/busybox && make -j4 && make install)
}

function build_initramfs {
    rm -rf build/initramfs
    rm -f build/initramfs.cpio.gz
    mkdir -p build/initramfs
    (cd build/initramfs \
        && mkdir -p proc sys \
        && cp -a ../busybox/_install/* ./ \
        && find . | cpio -o -c | gzip -9 > ../initramfs.cpio.gz)
}

case $1 in
linux)
    build_linux
    ;;
busybox)
    build_busybox
    ;;
initramfs)
    build_initramfs
    ;;
all|"")
    build_linux
    build_busybox
    build_initramfs
    ;;
*)
    echo "unexpected argument: `$1`" >&2
    exit 1
esac
