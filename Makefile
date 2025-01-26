disk_size := 1g

target := target/x86_64-unknown-lucy/debug
rom := $(target)/lucy.iso
initrd := $(target)/lucy.initrd
kernel := $(target)/lucy
lib_boot := $(target)/libboot.a
user_target := target/x86_64-unknown-none/release
user_program := $(user_target)/program
libc := $(target)

.PHONY: all
all: $(rom) $(target)/_disk_image

$(kernel): $(shell find kernel) $(lib_boot) boot/linker.ld
	cargo -Z unstable-options -C kernel build
$(user_program): $(shell find user)
	cargo -Z unstable-options -C user/program build --release
$(target)/boot.o: boot/boot.asm | $(target)
	nasm -felf64 $^ -o $@
$(target)/syscall.o: $(shell find kernel/src -name 'syscall.asm') | $(target)
	nasm -felf64 $< -o $@
$(target)/base: $(shell find base) $(target)
	cp -r $< $@
	mkdir -p $@/usr/bin
	cargo -Z unstable-options -C user/hello-world build --release
	cp $(user_target)/hello-world $@/usr/bin
$(lib_boot): $(target)/boot.o $(target)/syscall.o
	ar rvs $@ $^
$(initrd): $(user_program)
	cp $< $@
$(rom): $(kernel) $(initrd)
	./scripts/build_image.sh $(target)
$(target)/_disk_image: $(target)/base
	qemu-img create -f raw $@ $(disk_size)
	mkfs.ext2 -d $< $@
$(target):
	mkdir -p $@

.PHONY: run
run: $(rom) $(target)/_disk_image
	qemu-system-x86_64 -cdrom $(rom) \
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
