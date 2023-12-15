KERNEL_VMA equ 0xFFFFFF8000000000

extern _kernel_start
extern _kernel_end
extern _syscall_enter

BITS 32

;; Multiboot header.
section .multiboot
multiboot_start:
	;; Magic (double) word.
	dd 0x1BADB002
	;; ALIGN | MEMINFO | VIDEO
	dd 7
	;; Checksum
	dd -(7+0x1BADB002)
	dd 0,0,0,0,0

	;; Unused
	;; Linear framebuffer.
	dd 0
	;; Framebuffer width.
	dd 1024
	;; Framebuffer height.
	dd 768
	;; Color depth (bits).
	dd 32
multiboot_end:

section .bss

kernel_stack_bottom:
	align 0x1000
	resb  0x10000
kernel_stack_top:

section .text

global _start
_start:
	cli
stack_setup:
	;; Set stack pointer to the kernel stack top (physical).
	mov esp, kernel_stack_top - KERNEL_VMA
	xor ebp, ebp
	;; Assign multiboot arguments to soon to be 64 bit C calling convention
	;; registers.
	mov esi, ebx
	mov edi, eax
enable_paging:
	;; Set kernel page table.
	mov eax, boot_pml4 - KERNEL_VMA
	mov cr3, eax

	;; Set CPU flags for address extension (PAE [required for long mode]),
	;; and enable some instructions (rdfsbase, rdgsbase, etc.).
	mov eax, cr4
	or eax, 1 << 5
	or eax, 1 << 16
	mov cr4, eax

	;; EFER (enable long mode, enable syscall instruction).
	mov ecx, 0xC0000080
	rdmsr
	or eax, 1 << 0 ; syscall extensions
	or eax, 1 << 8 ; long mode enable
	or eax, 0x101
	wrmsr

	;; Enable paging.
	mov eax, cr0
	or eax, 1 << 31
	mov cr0, eax

	;; Load GDT, jump to long code.
	lgdt [gdt64.ptr - KERNEL_VMA]
	jmp 0x08:start_long_mode - KERNEL_VMA

BITS 64

extern kernel_start

start_long_mode:
	;; Jump to high virtual memory.
	mov rax, start_long_mode_high
	jmp rax
start_long_mode_high:
	;; Set stack to high virtual memory location.
	mov rsp, kernel_stack_top

	;; Adjust pointer to multiboot header to high address.
	mov rax, qword KERNEL_VMA
	add rsi, rax

	;; Unmap lower address range.
	mov rax, qword boot_pml4
	mov qword [rax], 0
	invlpg [0]

	;; Load GDT from high memory.
	mov rax, qword gdt64.ptr_high
	lgdt [rax]
	mov ax, gdt64.data
	mov ss, ax
	mov ds, ax
	mov es, ax
	mov gs, ax
	mov fs, ax

	;; Set STAR segment offsets
	mov ecx, 0xC0000081
	rdmsr
	xor edx, edx
	or edx, 0x0018 << 16 ; User CS+16:SS+8
	or edx, 0x0008 << 00 ; Kernel CS+0:SS+8
	wrmsr

	;; Set syscall handler via LSTAR.
	mov ecx, 0xC0000082
	rdmsr
	mov rax, _syscall_enter
	mov rdx, rax
	shr rdx, 32
	wrmsr

	;; Disable interrupts on syscall via FMASK.
	mov ecx, 0xC0000084
	rdmsr
	mov eax, 0x200
	wrmsr

	;; Call Rust main.
	call kernel_start
	ud2
idle:	hlt
	jmp idle

	global outsl_asm
	;; (rdi: len, rsi: src, rdx: port)
outsl_asm:
	push rbp
	mov rbp, rsp

	push rcx
	mov rcx, rdi

	rep outsd

	pop rcx

	pop rbp
	ret

	global insl_asm
	;; (rdi: dst, rsi: len, rdx: port)
insl_asm:
	push rbp
	mov rbp, rsp

	push rcx
	mov rcx, rsi

	rep insd

	pop rcx

	pop rbp
	ret

section .data

align 0x1000
boot_pd:
	dq 0x83
	dq 0x200083
	dq 0x400083
	dq 0x600083
	dq 0x800083
	times 507 dq 0
boot_pdp:
	dq boot_pd + 0x3 - KERNEL_VMA
	times 511 dq 0
global boot_pml4
boot_pml4:
	dq boot_pdp + 0x3 - KERNEL_VMA
	times 510 dq 0
	dq boot_pdp + 0x3 - KERNEL_VMA

global boot_gdt
boot_gdt:
gdt64:
.kernel:              ; 0
	dq 0
.code: equ $ - gdt64  ; 8
	dq (0x9A << 40) | (1 << 53)
.data:	equ $ - gdt64 ; 10
	dq (0x92 << 40)
.user: equ $ - gdt64  ; 18
	dq 0
.udata:	equ $ - gdt64 ; 20
	dq (0xF2 << 40)
.ucode:	equ $ - gdt64 ; 28
	dq (0xFA << 40) | (1<<53)
.tss:   equ $ - gdt64 ; 30
	dq 0
	dq 0
.ptr:
	dw $ - gdt64 - 1
	dq gdt64 - KERNEL_VMA
.ptr_high:
	dw .ptr - gdt64 - 1
	dq gdt64

global boot_tss
boot_tss:
	;; Reserved
	dd 0
	;; RSP0
	dq kernel_stack_top
	;; RSP1-END
	times 23 dd 0
