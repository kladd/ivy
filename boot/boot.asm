BITS 32

section .multiboot
multiboot_start:
	dd 0x1BADB002                  ; MULTIBOOT_MAGIC
	dd 7                           ; ALIGN | MEMINFO
	dd -(7+0x1BADB002)             ; CHECKSUM
	dd 0,0,0,0,0                   ; Unused.
	dd 0
	dd 1024
	dd 768
	dd 32
multiboot_end:

section .bss
kernel_stack_bottom:
	align 4096
	resb 16384
kernel_stack_top:
align 4096
global pml4_table
pml4_table:
	resb 4096
global pdp_table
pdp_table:
	resb 4096
global pd_table
pd_table:
	resb 4096

global _start
section .text
_start:                                ; The universe begins here.
	cli
stack_setup:                           ; Create first kernel stack.
	mov esp, kernel_stack_top
	xor ebp, ebp
	mov esi, ebx                   ; Save multiboot magic + flags.
	mov edi, eax                   ; (in long mode these will be arguments)
enable_paging:
	mov eax, boot_pml4             ; Set pml4 address in cr3
	mov cr3, eax

	mov eax, cr4                   ; Set PAE flag
	or eax, 1 << 5
	mov cr4, eax

	mov ecx, 0xC0000080            ; Set long mode
	rdmsr
	or eax, 1 << 8
	wrmsr

	mov eax, cr0                   ; Enable paging.
	or eax, 1 << 31
	mov cr0, eax

	lgdt [gdt64.ptr]
	jmp GDT64_CS:start_long_mode

section .data
align 0x1000
boot_pd:
	times 4 dq 0x83
	times 508 dq 0
boot_pdp:
	dq boot_pd + 0x3
	times 511 dq 0
boot_pml4:
	dq boot_pdp + 0x3
	times 511 dq 0

section .rodata
gdt64:
	dq 0
.code:
	dq (1<<43) | (1<<44) | (1<<47) | (1<<53)
.ptr:
	dw $ - gdt64 - 1
	dq gdt64
GDT64_CS equ gdt64.code - gdt64

BITS 64
section .text
extern kernel_start
start_long_mode:
	mov ax, 0                      ; Zero out segment registers
	mov ss, ax
	mov ds, ax
	mov es, ax
	mov gs, ax
	mov fs, ax

	call kernel_start
idle:	hlt
	jmp idle
