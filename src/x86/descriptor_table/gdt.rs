use core::arch::asm;

use crate::x86::descriptor_table::DescriptorTableRegister;

#[derive(Default)]
#[repr(packed)]
pub struct SegmentDescriptor {
	limit_low: u16,
	base_low: u16,
	base_mid: u8,
	access: u8,
	limit_high_flags: u8,
	base_high: u8,
}

impl SegmentDescriptor {
	pub fn null() -> Self {
		Self::default()
	}

	pub fn new(limit: u32, base: u32, access: u8, flags: u8) -> Self {
		let mut desc = Self::null();

		desc.base_low = base as u16;
		desc.base_mid = (base >> 16) as u8;
		desc.base_high = (base >> 24) as u8;

		desc.limit_low = limit as u16;

		desc.limit_high_flags = ((limit >> 16) & 0x0F) as u8;
		desc.limit_high_flags |= flags & 0xF0;

		desc.access = access;

		desc
	}
}

#[allow(named_asm_labels)]
pub unsafe fn flush(dtr: &DescriptorTableRegister) {
	asm!(
		r#"
			lgdt [eax]

			mov ax, 0x10
			mov ds, ax
			mov es, ax
			mov fs, ax
			mov gs, ax
			mov ss, ax

			jmp 0x08, offset .resume
		.resume:"#,
		in("eax") dtr,
	);
}
