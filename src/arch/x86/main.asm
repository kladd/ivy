BITS 32

extern _multiboot
extern _code
extern _bss
extern _end
extern kernel_start
extern print_exception_stack_frame
extern handle_interval_timer

section .multiboot
header_start:
	dd 0x1BADB002      ; MAGIC
	dd 3               ; ALIGN | MEMINFO
	dd -(3+0x1BADB002) ; CHECKSUM

	dd _multiboot
	dd _code
	dd _bss
	dd _end
	dd _start
header_end:

section .bss
align 16
stack_bottom:
	resb 16384
stack_top:

section .text
global _start
_start:
	mov esp, stack_top
	push ebx
	push eax
	cli
	call kernel_start
_halt:
	hlt
	jmp _halt

global unimplemented_interrupt_handler
unimplemented_interrupt_handler:
	cli
	pushad
	mov eax, esp
	add eax, 32
	push eax
	call print_exception_stack_frame
	pop eax
	popad
	sti
	iret

global interval_timer_handler
interval_timer_handler:
	cli
	pushad
	call handle_interval_timer
	popad
	sti
	iret