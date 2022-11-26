kernel := target/kernel/ivy

all: $(kernel)

$(kernel): always
	@cargo build

run: $(kernel)
	@qemu-system-i386 -kernel $< -serial stdio

always: ;
