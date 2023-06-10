	bits 64

	extern syscall_enter
	global _syscall_enter
	global _syscall_ret

	section .text
_syscall_enter:
	swapgs
	mov [gs:0 + 8], rsp
	mov rsp, [gs:0]

	call syscall_enter
	int 0x3
	hlt

_syscall_ret:
	cli
	rdgsbase r11
	mov [r11], rsp
	mov rsp, [r11 + 8]
	mov r11, 0x202
	swapgs
	o64 sysret