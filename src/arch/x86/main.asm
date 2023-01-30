BITS 32

extern _multiboot
extern _code
extern _bss
extern _end
extern kernel_start
extern print_exception_stack_frame

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

global enable_paging
enable_paging:
    push eax

    mov eax, cr0
    or eax, 0x80000000
    mov cr0, eax

    pop eax
    ret