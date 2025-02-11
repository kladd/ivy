SECTIONS
{
	. = 0x200000;
	/DISCARD/ : {
		*(.comment)
		*(.gnu*)
		*(.note*)
		*(.eh_frame_hdr) *(.eh_frame_entry .eh_frame_entry.*) *(.eh_frame) *(.eh_frame.*)
	}
}
