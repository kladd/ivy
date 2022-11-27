	.section .text
	.global _start
_start:
	push ebx
	push eax
	call kernel_start
	cli
_halt:
	hlt
	jmp _halt