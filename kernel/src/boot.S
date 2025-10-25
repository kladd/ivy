.extern kernel_start

.section .text.boot

.global _start
_start:
	// Use CPU 0 for now
	mrs  x5, mpidr_el1
	and  x5, x5, #0x3
	cbz  x5, __continue
	b    _cpu_halt

__continue:
	// Init stack pointer
	ldr  x5, =_start
	mov  sp, x5

	// Jump to kernel
	bl   kernel_start

.global _cpu_halt
_cpu_halt:
	movz x2, #0xfbad
	movk x2, #0xdeca, lsl #16
	wfi
	b    _cpu_halt
