KERNEL_VMA equ 0xffffffff80000000

extern _kernel_start
extern _kernel_end

BITS 32

section .multiboot
multiboot_start:
	dd 0x1BADB002                  ; MULTIBOOT_MAGIC
	dd 7                           ; ALIGN | MEMINFO
	dd -(7+0x1BADB002)             ; CHECKSUM
	dd 0,0,0,0,0
	dd 0                           ; Linear buffer
	dd 1024                        ; width
	dd 768                         ; height
	dd 32                          ; color depth
multiboot_end:

section .bss
kernel_stack_bottom:
	align 4096
	resb 16384
kernel_stack_top:

global _start
section .text
_start:                                ; The universe begins here.
	cli
stack_setup:                           ; Create first kernel stack.
	mov esp, kernel_stack_top - KERNEL_VMA
	xor ebp, ebp
	mov esi, ebx                   ; Save multiboot magic + flags.
	mov edi, eax                   ; (in long mode these will be arguments)
enable_paging:
        ; Set pml4 address in cr3
	mov eax, boot_pml4 - KERNEL_VMA
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

	lgdt [gdt64.ptr - KERNEL_VMA]
	jmp 0x08:start_long_mode - KERNEL_VMA

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

	mov rax, start_long_mode_high  ; abs jump to high memory
	jmp rax
start_long_mode_high:
	mov rsp, kernel_stack_top      ; set the stack back to high memory
	add rsi, KERNEL_VMA            ; adjust the multiboot header to high
	                               ; mem also

	mov qword [boot_pml4], 0       ; unmap low memory
	mov qword [boot_pdp], 0        ;
	invlpg [0]                     ;
	lgdt [gdt64_high.ptr]          ; load a GDT in high space

	call kernel_start
idle:	hlt
	jmp idle

section .data
align 0x1000
boot_pd:
	times 4 dq 0x83
	times 508 dq 0
boot_pdp:
	dq boot_pd + 0x3 - KERNEL_VMA
	times 509 dq 0
	dq boot_pd + 0x3 - KERNEL_VMA
	dq 0
boot_pml4:
	dq boot_pdp + 0x3 - KERNEL_VMA
	times 510 dq 0
	dq boot_pdp + 0x3 - KERNEL_VMA

section .rodata
gdt64:
	dq 0
.code:
	dq (1<<43) | (1<<44) | (1<<47) | (1<<53)
.ptr:
	dw $ - gdt64 - 1
	dq gdt64 - KERNEL_VMA

gdt64_high:
	dq 0
.code:
	dq (1<<43) | (1<<44) | (1<<47) | (1<<53)
.ptr:
	dw $ - gdt64 - 1
	dq gdt64

