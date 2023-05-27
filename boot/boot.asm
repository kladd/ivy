KERNEL_VMA equ 0xffffffff80000000

extern _kernel_start
extern _kernel_end
extern handle_syscall

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
	align 0x1000
	resb  0x4000
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

	mov ecx, 0xC0000080            ; Set long mode & system call extensions.
	rdmsr
	or eax, 0x101
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

	mov rax, start_long_mode_high  ; abs jump to high memory
	jmp rax
start_long_mode_high:
	mov rsp, kernel_stack_top      ; set the stack back to high memory
	add rsi, KERNEL_VMA            ; adjust the multiboot header to high
	                               ; mem also

	mov qword [boot_pml4], 0       ; unmap low memory
	mov qword [boot_pdp], 0        ;
	invlpg [0]                     ;
	lgdt [gdt64.ptr_high]          ; load a GDT in high space
resume:
	mov ax, gdt64.data
	mov ss, ax
	mov ds, ax
	mov es, ax
	mov gs, ax
	mov fs, ax

init_syscalls:
	mov ecx, 0xC0000081            ; Set STAR CS:SS
	rdmsr
	mov edx, 0x00180008
	wrmsr

	mov ecx, 0xC0000082            ; Set LSTAR
	rdmsr
	mov rax, handle_syscall
	mov rdx, rax
	shr rdx, 32
	wrmsr

	mov ecx, 0xC0000084            ; Set FMASK (disable interrupts on syscall)
	rdmsr
	mov eax, 0x200
	wrmsr

	call kernel_start
idle:	hlt
	jmp idle

section .data
align 0x1000
boot_pd:
	times 4 dq 0x87
	times 508 dq 0
boot_pdp:
	dq boot_pd + 0x3 - KERNEL_VMA
	times 509 dq 0
	dq boot_pd + 0x7 - KERNEL_VMA
	dq 0
boot_pml4:
	dq boot_pdp + 0x3 - KERNEL_VMA
	times 510 dq 0
	dq boot_pdp + 0x7 - KERNEL_VMA

gdt64:
.kernel:              ; 0
	dq 0
.code: equ $ - gdt64  ; 8
	dq (0x9A << 40) | (1 << 53)
.data:	equ $ - gdt64 ; 10
	dq (0x92 << 40)
.user: equ $ - gdt64  ; 18
	dq 0
.udata:	equ $ - gdt64 ; 20             ; STAR expects selectors in order of data,code
	dq (0xF2 << 40)
.ucode:	equ $ - gdt64 ; 28
	dq (0xFA << 40) | (1<<53)
.ptr:
	dw $ - gdt64 - 1
	dq gdt64 - KERNEL_VMA
.ptr_high:
	dw .ptr - gdt64 - 1
	dq gdt64
