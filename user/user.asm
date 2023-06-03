	bits 64

	section .text
	global _start
_start:
	mov r10, 0xdeadbeef
	o64 syscall