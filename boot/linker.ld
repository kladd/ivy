ENTRY(_start)

SECTIONS
{
	. = 1M;

	_kernel_start = .;
	.boot :
	{
		KEEP(*(.multiboot .multiboot.*))
	}

	.text ALIGN(4K) :
	{
		*(.text)
	}
	_kernel_end = .;
}