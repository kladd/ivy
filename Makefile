target_dir := target/kernel
start_obj := $(target_dir)/start.o
start_a := $(target_dir)/libstart.a
kernel := $(target_dir)/ivy
disk_size := 1g

# Append .exe to qemu commands if we're in WSL.
qemu_exe := "$(shell cat /proc/version | grep -q microsoft && echo ".exe")"

all: $(kernel)

$(target_dir):
	mkdir -p $@

$(start_obj): src/arch/x86/main.asm $(target_dir)
	nasm -o $@ -felf32 $<

$(start_a): $(start_obj)
	ar rvs $@ $^

$(kernel): $(start_a) always
	@cargo build

$(target_dir)/_disk_image: $(kernel)
	qemu-img$(qemu_exe) create -f raw $@ $(disk_size)
	mkfs.fat -F 16 $@
	mkdir -p $(target_dir)/mnt
	sudo mount $@ $(target_dir)/mnt
	sudo cp -r base/* $(target_dir)/mnt
	sudo umount $(target_dir)/mnt

run: $(kernel) $(target_dir)/_disk_image
	@qemu-system-i386$(qemu_exe) -kernel $< \
		-m 2g \
		-serial stdio \
		-drive file=$(target_dir)/_disk_image,format=raw,media=disk

always: ;

.PHONY: clean
clean:
	cargo clean
