set -e

qemu-system-x86_64 \
    -kernel build/linux/arch/x86_64/boot/bzImage \
    -initrd build/initramfs.cpio.gz \
    -nographic \
    -append console=ttyS0 \
    "$@"
