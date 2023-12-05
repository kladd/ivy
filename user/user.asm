	bits 64

	section .text
	global _start
_start:
	mov rax, 400
	o64 syscall
	mov rax, 402
	o64 syscall
	mov rax, 400
	o64 syscall
	mov rax, 60
	o64 syscall
