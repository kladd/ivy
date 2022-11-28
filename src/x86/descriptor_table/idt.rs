use core::arch::asm;

use crate::x86::descriptor_table::DescriptorTableRegister;

#[derive(Default, Copy, Clone)]
#[repr(packed)]
pub struct InterruptDescriptor {
	isr_low: u16,
	kernel_cs: u16,
	_zero: u8,
	flags: u8,
	isr_high: u16,
}

impl InterruptDescriptor {
	pub fn null() -> Self {
		Self::default()
	}

	pub fn new(isr: u32, sel: u16, flags: u8) -> Self {
		let mut desc = Self::default();

		desc.isr_low = isr as u16;
		desc.isr_high = (isr >> 16) as u16;
		desc.kernel_cs = sel;
		desc.flags = flags;

		desc
	}
}

pub unsafe fn flush(dtr: &DescriptorTableRegister) {
	asm!("lidt [eax]", in("eax") dtr)
}
