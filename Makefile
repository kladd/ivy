target_dir := target/kernel
start_obj := $(target_dir)/start.o
start_a := $(target_dir)/libstart.a
kernel := $(target_dir)/ivy

all: $(kernel)

$(target_dir):
	mkdir -p $@

$(start_obj): src/arch/x86/main.asm $(target_dir)
	nasm -o $@ -felf32 $<

$(start_a): $(start_obj)
	ar rvs $@ $^

$(kernel): $(start_a) always
	@cargo build

run: $(kernel)
	@qemu-system-i386 -kernel $< -m 2g -serial stdio

always: ;

.PHONY: clean
clean:
	cargo clean
