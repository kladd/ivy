# Requires:
#  - nasm
#  - xorriso
#  - grub-core
#  - grub-pc-bin
#  - qemu
#  - ld
#  - rustc

tgt := target
rom := $(tgt)/lucy.rom
exe := "$(shell cat /proc/version | grep -q microsoft && echo ".exe")"
kernel := $(tgt)/x86_64-unknown-lucy/debug/lucy
boot_lib := $(tgt)/libboot.a
initrd := $(tgt)/lucy.initrd
disk_size := 1g

all: $(rom)
$(tgt):
	mkdir -p $(tgt)

# ASM files are packaged into libboot.a and statically linked with the kernel
# executable.
$(tgt)/boot.o: boot/boot.asm | $(tgt)
	nasm -felf64 $^ -o $@
$(tgt)/syscall.o: src/arch/amd64/syscall.asm | $(tgt)
	nasm -felf64 $^ -o $@
$(boot_lib): $(tgt)/boot.o $(tgt)/syscall.o
	ar rvs $@ $^

# Kernel binary.
$(kernel): always $(boot_lib)
	cargo build

# Test user program is assembled without linking, loaded into ramdisk by GRUB.
$(initrd): user/user.asm
	mkdir -p $(tgt)
	nasm -o $@ $<

# Kernel boots from ROM.
# TODO: A font.
$(rom): boot/grub.cfg $(kernel) $(initrd)
	mkdir -p $(tgt)/rom/boot/grub
	cp boot/grub.cfg $(tgt)/rom/boot/grub/grub.cfg
	cp $(kernel) $(tgt)/rom/boot/lucy
	cp $(initrd) $(tgt)/rom/boot/lucy.initrd
	gunzip -c /usr/share/kbd/consolefonts/sun12x22.psfu.gz > $(tgt)/rom/boot/font.psfu
	grub-mkrescue -o $(rom) $(tgt)/rom

$(tgt)/_disk_image: base
	qemu-img$(exe) create -f raw $@ $(disk_size)
	mkfs.ext2 -d base $@

run: $(rom) $(tgt)/_disk_image
	qemu-system-x86_64$(exe) -cdrom $(rom) \
		-cpu Broadwell \
		-drive file=$(tgt)/_disk_image,format=raw,if=ide \
		-d int \
		-m 2g \
		-no-reboot \
		-no-shutdown \
		-serial stdio
.PHONY: clean
clean:
	$(RM) -r $(rom) $(tgt)/boot.o $(tgt)/boot.elf $(tgt)/rom
	cargo clean

.PHONY: always
always: ;