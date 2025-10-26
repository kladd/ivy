.extern kernel_start
.extern DEBUG_VALUE
.extern ROM_END

//----------------------------------------------------------------------------
.section ".text.boot"

.global _start
_start:
	// Init stack pointer
	ldr     x5, ROM_END
	mov     sp, x5

	// clear bss
	ldr     x5, =__bss_start
	ldr     w6, =__bss_size
__clear_bss:
	cbz     w6, __start_kernel
	str     xzr, [x5], #8
	sub     w6, w6, #1
	cbnz    w6, __clear_bss

__start_kernel:
	// Register exception vectors
	adr     x0, __exception_vector_table_el1
	msr     vbar_el1, x0

	// Enable FP/SIMD
	mov     x0, #(0x3 << 20)
	msr     CPACR_EL1, x0
	isb

	b       kernel_start

.global _cpu_halt
_cpu_halt:
	isb
	wfi
	b       _cpu_halt

//----------------------------------------------------------------------------
.section .text.vectors

.macro exception_entry label
	.balign 0x80
	b       \label
.endm

.global __exception_vector_table_el1
.balign 0x800
__exception_vector_table_el1:
	exception_entry	__exception_handler_el1
	exception_entry	__exception_handler_el1
	exception_entry	__exception_handler_el1
	exception_entry	__exception_handler_el1

	exception_entry	__exception_handler_el1
	exception_entry	__exception_handler_el1
	exception_entry	__exception_handler_el1
	exception_entry	__exception_handler_el1

	exception_entry	__exception_handler_el1
	exception_entry	__exception_handler_el1
	exception_entry	__exception_handler_el1
	exception_entry	__exception_handler_el1

	exception_entry	__exception_handler_el1
	exception_entry	__exception_handler_el1
	exception_entry	__exception_handler_el1
	exception_entry	__exception_handler_el1
__exception_handler_el1:
	ldr     x14, DEBUG_VALUE
	mrs     x17, elr_el1
	mrs     x20, spsr_el1
	mrs     x23, esr_el1
	mrs     x26, far_el1
	b       _cpu_halt
