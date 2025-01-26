#!/usr/bin/env bash

set -ex

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)

target_path=$SCRIPT_DIR/../$1
kernel_path=$SCRIPT_DIR/../$1/lucy
initrd_path=$SCRIPT_DIR/../$1/lucy.initrd
conf_path=$SCRIPT_DIR/../boot/limine.conf

cd "$SCRIPT_DIR"/../target

[ ! -d limine ] &&
    git clone https://github.com/limine-bootloader/limine.git --branch=v8.x-binary --depth=1
make -C limine

mkdir -p iso_root/boot
cp -v "$kernel_path" iso_root/boot/
cp -v "$initrd_path" iso_root/boot/
mkdir -p iso_root/boot/limine
cp -v "$conf_path" limine/limine-bios.sys limine/limine-bios-cd.bin \
      limine/limine-uefi-cd.bin iso_root/boot/limine/

mkdir -p iso_root/EFI/BOOT
cp -v limine/BOOTX64.EFI iso_root/EFI/BOOT/
cp -v limine/BOOTIA32.EFI iso_root/EFI/BOOT/

xorriso -as mkisofs -R -r -J -b boot/limine/limine-bios-cd.bin \
    -no-emul-boot -boot-load-size 4 -boot-info-table -hfsplus \
    -apm-block-size 2048 --efi-boot boot/limine/limine-uefi-cd.bin \
    -efi-boot-part --efi-boot-image --protective-msdos-label \
    iso_root -o "$target_path"/lucy.iso

./limine/limine bios-install "$target_path"/lucy.iso
