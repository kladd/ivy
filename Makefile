target_dir := target/kernel
start_obj := $(target_dir)/start.o
start_a := $(target_dir)/libstart.a
kernel := $(target_dir)/ivy
disk_size := 1g
log_level := debug

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
	cargo build --features log/max_level_$(log_level)

.PHONY: run
run: $(kernel)
	@qemu-system-i386$(qemu_exe) -kernel $< \
		-m 2g \
		-serial stdio \
		-no-reboot \
		-no-shutdown \
		-drive file=fat:rw:base,format=raw,media=disk,cache=writethrough

# TODO: Headless as option
.PHONY: headless
headless: $(kernel)_headless
	@qemu-system-i386$(qemu_exe) -kernel $(kernel) \
		-m 2g \
		-nographic \
		-no-reboot \
		-no-shutdown \
		-drive file=fat:rw:base,format=raw,media=disk,cache=writethrough

.PHONY: $(kernel)_headless
$(kernel)_headless: $(start_a)
	cargo build --features log/max_level_$(log_level),headless


always: ;

.PHONY: clean
clean:
	cargo clean
