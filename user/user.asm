	bits 64

	section .text
	global _start
_start:
    mov rax, 400
	o64 syscall
	o64 syscall
    mov rax, 401
	o64 syscall
	ud2