	bits 64

	extern syscall_enter
	global _syscall_enter
	global _syscall_ret

	section .text
_syscall_enter:
	swapgs
	mov [gs:0 + 8], rsp
	mov rsp, [gs:0]
	sti

	push rcx ; rip
	push r15
	push r14
	push r13
	push r12
	push r11
	push r10
	push r9
	push r8
	push rax
	push rcx
	push rdx
	push rbx
	push rsp
	push rbp
	push rsi
	push rdi

	mov rdi, rsp
	call syscall_enter

	pop rdi
	pop rsi
	pop rbp
	add rsp, 8
	pop rbx
	pop rdx
	pop rcx
	pop rax
	pop r8
	pop r9
	pop r10
	pop r11
	pop r12
	pop r13
	pop r14
	pop r15
	pop rcx ; rip
_syscall_ret:
	cli
	rdgsbase r11
	mov [r11], rsp
	mov rsp, [r11 + 8]
	mov r11, 0x202
	swapgs
	o64 sysret
