# Requires:
#  - nasm
#  - xorriso
#  - grub-core
#  - grub-pc-bin
#  - qemu
#  - ld

tgt := target
rom := $(tgt)/lucy.rom
exe := "$(shell cat /proc/version | grep -q microsoft && echo ".exe")"
kernel := $(tgt)/x86_64-unknown-lucy/debug/lucy
boot_lib := $(tgt)/libboot.a
initrd := $(tgt)/lucy.initrd

all: $(rom)

$(tgt)/boot.o: boot/boot.asm
	mkdir -p $(tgt)
	nasm -felf64 $< -o $@
$(tgt)/syscall.o: src/arch/amd64/syscall.asm
	mkdir -p $(tgt)
	nasm -felf64 $< -o $@
$(initrd): user/user.asm
	mkdir -p $(tgt)
	nasm -o $@ $<
$(rom): boot/grub.cfg $(kernel) $(initrd)
	mkdir -p $(tgt)/rom/boot/grub
	cp boot/grub.cfg $(tgt)/rom/boot/grub/grub.cfg
	cp $(kernel) $(tgt)/rom/boot/lucy
	cp $(initrd) $(tgt)/rom/boot/lucy.initrd
	grub-mkrescue -o $(rom) $(tgt)/rom
$(kernel): always $(boot_lib)
	cargo build
$(boot_lib): $(tgt)/boot.o $(tgt)/syscall.o
	ar rvs $@ $^
run: $(rom)
	qemu-system-x86_64$(exe) -cdrom $(rom) \
		-cpu qemu64,+fsgsbase \
		-m 2g \
		-no-reboot \
		-no-shutdown \
		-d int \
		-serial stdio
.PHONY: clean
clean:
	$(RM) -r $(rom) $(tgt)/boot.o $(tgt)/boot.elf $(tgt)/rom
	cargo clean

.PHONY: always
always: ;