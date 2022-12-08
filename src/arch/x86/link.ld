ENTRY(_start)

SECTIONS
{
	. = 0x100000;

	.boot :
	{
		_multiboot = .;
		KEEP(*(.multiboot));
	}

	.text : ALIGN(4K)
	{
		_code = .;
		*(.text .text.*);
	}

	.data : ALIGN(4K)
	{
		_data = .;
		data = .;
	}

	.bss : ALIGN(4K)
	{
		_bss = .;
		KEEP(*(.bss));
	}

	_end = .;
}