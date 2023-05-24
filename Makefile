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
kernel := $(tgt)/x86_64-unknown-lucy/debug/liblucy.a

all: $(rom)

$(tgt)/boot.o: boot/boot.asm
	nasm -felf64 $< -o $@
$(tgt)/boot.elf: boot/linker.ld $(tgt)/boot.o $(kernel)
	ld -n -o $@ -T $< $(tgt)/boot.o $(kernel)
$(rom): boot/grub.cfg $(tgt)/boot.elf
	mkdir -p $(tgt)/rom/boot/grub
	cp boot/grub.cfg $(tgt)/rom/boot/grub/grub.cfg
	cp $(tgt)/boot.elf $(tgt)/rom/boot/boot.elf
	grub-mkrescue -o $(rom) $(tgt)/rom
$(kernel): always
	cargo build
run: $(rom)
	qemu-system-x86_64$(exe) -cdrom $(rom) \
		-m 2g \
		-no-reboot \
		-no-shutdown \
		-serial stdio
.PHONY: clean
clean:
	$(RM) -r $(rom) $(tgt)/boot.o $(tgt)/boot.elf $(tgt)/rom

.PHONY: always
always: ;