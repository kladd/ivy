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

global insl_asm
insl_asm:
	push ebp
	mov ebp, esp

	push ecx
	push edi
	push eax

	mov ecx, [ebp + 16] ; third argument, length in ecx
	mov edi, [ebp + 12] ; second argument (out) in edi
	mov dx, [ebp + 8] ; first argument (port) in dx

	rep insd

	pop eax
	pop edi
	pop ecx

	pop ebp
	ret

;; fn switch_task(task: &Task)
;; TODO: lol can't switch back
global switch_task
switch_task:
	push ebx
	push esi
	push edi
	push ebp

	mov esi, [esp + 20] ; task

	;; Switch to new task's stack.
	mov esp, [esi + 4]  ; task.esp

	;; TODO: All tasks use kernel PD right now.
	;; Switch to new task's page directory.
	;; mov eax, [esi + 8]  ; task.cr3
	;; mov cr3, eax

	;; TODO: All tasks are kernel mode right now.
	;; Update TSS (Ring 3 -> Ring 0)
	;; mov ebx, [esi + 12] ; task.esp0
	;; mov esp0 into TSS

	pop ebp
	pop edi
	pop esi
	pop ebx

	ret ;; New task EIP popped from its kernel stack.
