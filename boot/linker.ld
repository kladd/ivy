ENTRY(_start)

KERNEL_LMA = 0x0000000000100000;
KERNEL_VMA = 0xFFFFFF7F80000000; /* PML4[510][510][0] */

SECTIONS
{
	. = KERNEL_VMA + KERNEL_LMA;

	_kernel_start = .;
	.boot : AT(ADDR(.boot) - KERNEL_VMA)
	{
		KEEP(*(.multiboot))
	}

	.text ALIGN(4K) : AT(ADDR(.text) - KERNEL_VMA)
	{
		*(.text .text*)
	}

	.bss ALIGN(4K) : AT(ADDR(.bss) - KERNEL_VMA)
	{
		*(.bss .bss*)
	}

	.data ALIGN(4K) : AT(ADDR(.data) - KERNEL_VMA)
	{
		*(.data .data*)
	}

	.rodata ALIGN(4K) : AT(ADDR(.rodata) - KERNEL_VMA)
	{
		*(.rodata .rodata*)
	}
	_kernel_end = .;
}