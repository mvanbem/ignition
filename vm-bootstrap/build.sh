#!/bin/bash
set -e
set -x

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

function build_gcc {
    rm -rf packages/{gcc-10.2.0,gmp-6.2.1,mpfr-4.1.0,mpc-1.2.1}
    rm -rf build/gcc
    (cd packages \
        && tar xf gcc-10.2.0.tar.xz \
        && tar xf gmp-6.2.1.tar.xz \
        && tar xf mpfr-4.1.0.tar.xz \
        && tar xf mpc-1.2.1.tar.gz)
    (cd packages/gcc-10.2.0 \
        && mv ../gmp-6.2.1 gmp \
        && mv ../mpfr-4.1.0 mpfr \
        && mv ../mpc-1.2.1 mpc)
    mkdir -p build/gcc
    (cd build/gcc \
        && ../../packages/gcc-10.2.0/configure \
            --prefix=/usr \
            --disable-multilib \
            --enable-languages=c \
        && make -j4)
}

case $1 in
linux) build_linux;;
busybox) build_busybox;;
initramfs) build_initramfs;;
gcc) build_gcc;;
all|"")
    build_linux
    build_busybox
    build_initramfs
    build_gcc
    ;;
*)
    echo "unexpected argument: \`$1\`" >&2
    exit 1
esac
