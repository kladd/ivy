exe := "$(shell cat /proc/version | grep -q microsoft && echo ".exe")"
disk_size := 1g

target := target/x86_64-unknown-lucy/debug
rom := $(target)/lucy.iso
initrd := $(target)/lucy.initrd
kernel := $(target)/lucy
lib_boot := $(target)/libboot.a
user_program := target/x86_64-unknown-none/debug/program

.PHONY: all
all: $(rom)

$(kernel): $(shell find kernel) $(lib_boot) boot/linker.ld
	cargo -Z unstable-options -C kernel build
$(user_program): $(shell find user)
	cargo -Z unstable-options -C user/program build
$(target)/boot.o: boot/boot.asm | $(target)
	nasm -felf64 $^ -o $@
$(target)/syscall.o: $(shell find kernel/src -name 'syscall.asm') | $(target)
	nasm -felf64 $< -o $@
$(lib_boot): $(target)/boot.o $(target)/syscall.o
	ar rvs $@ $^
$(initrd): $(user_program)
	cp $< $@
$(rom): $(kernel) $(initrd)
	mkdir -p $(target)/rom/boot/grub
	cp boot/grub.cfg $(target)/rom/boot/grub/grub.cfg
	cp $(kernel) $(target)/rom/boot/lucy
	cp $(initrd) $(target)/rom/boot/lucy.initrd
	gunzip -c /usr/share/kbd/consolefonts/sun12x22.psfu.gz > $(target)/rom/boot/font.psfu
	grub-mkrescue -o $@ $(target)/rom
$(target)/_disk_image: $(shell find base)
	qemu-img$(exe) create -f raw $@ $(disk_size)
	mkfs.ext2 -d base $@
$(target):
	mkdir -p $@

.PHONY: run
run: $(rom) $(target)/_disk_image
	qemu-system-x86_64$(exe) -cdrom $(rom) \
		-cpu Broadwell \
		-drive file=$(target)/_disk_image,format=raw,if=ide \
		-m 2g \
		-no-reboot \
		-no-shutdown \
		-serial stdio

.PHONY: clean
clean:
	$(RM) -r $(rom) $(target)/boot.o $(target)/syscall.o $(target)/rom
	cargo clean

