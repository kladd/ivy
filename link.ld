ENTRY(_start)

SECTIONS
{
	. = 0x100000;

	.boot :
	{
		KEEP(*(.multiboot));
	}

	.text : ALIGN(4K)
	{
		*(.text .text.*);
	}
}