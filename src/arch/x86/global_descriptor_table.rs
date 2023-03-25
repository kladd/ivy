use core::arch::asm;

use crate::arch::x86::descriptor_table::DescriptorTableRegister;

const SEGMENT_DESCRIPTOR_COUNT: usize = 5;
static mut DESCRIPTOR_TABLE: [SegmentDescriptor; SEGMENT_DESCRIPTOR_COUNT] =
	[SegmentDescriptor::null(); SEGMENT_DESCRIPTOR_COUNT];

#[derive(Copy, Clone)]
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
	pub const fn null() -> Self {
		Self {
			limit_low: 0,
			base_low: 0,
			base_mid: 0,
			access: 0,
			limit_high_flags: 0,
			base_high: 0,
		}
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

pub fn init_gdt() {
	unsafe {
		DESCRIPTOR_TABLE[1] = SegmentDescriptor::new(0xFFFFFFFF, 0, 0x9A, 0xCF);
		DESCRIPTOR_TABLE[2] = SegmentDescriptor::new(0xFFFFFFFF, 0, 0x92, 0xCF);
		DESCRIPTOR_TABLE[3] = SegmentDescriptor::new(0xFFFFFFFF, 0, 0xFA, 0xCF);
		DESCRIPTOR_TABLE[4] = SegmentDescriptor::new(0xFFFFFFFF, 0, 0xF2, 0xCF);
		flush_gdt();
	}
}

#[allow(named_asm_labels)]
fn flush_gdt() {
	// TODO: Don't hard-code segment offsets.
	unsafe {
		let gdtr = DescriptorTableRegister::new(&DESCRIPTOR_TABLE);
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
			in("eax") &gdtr,
		);
	}
}
