use core::arch::asm;

use crate::arch::x86::descriptor_table::DescriptorTableRegister;

const MAX_INTERRUPTS: usize = 256;

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct InterruptDescriptor {
	isr_low: u16,
	kernel_cs: u16,
	_zero: u8,
	flags: u8,
	isr_high: u16,
}

impl InterruptDescriptor {
	pub const fn null() -> Self {
		Self {
			isr_low: 0,
			kernel_cs: 0,
			_zero: 0,
			flags: 0,
			isr_high: 0,
		}
	}

	pub fn new(isr: u32, sel: u16, flags: u8) -> Self {
		let mut desc = Self::null();

		desc.isr_low = isr as u16;
		desc.isr_high = (isr >> 16) as u16;
		desc.kernel_cs = sel;
		desc.flags = flags;

		desc
	}
}

pub fn init_idt() -> [InterruptDescriptor; MAX_INTERRUPTS] {
	let mut idt = [InterruptDescriptor::null(); MAX_INTERRUPTS];

	for i in 0..32 {
		idt[i] = InterruptDescriptor::new(
			unimplemented_interrupt as u32,
			0x08,
			0x8E,
		);
	}

	idt
}

pub fn flush_idt(idt: &[InterruptDescriptor; MAX_INTERRUPTS]) {
	let idtr = DescriptorTableRegister::new(&idt);
	unsafe {
		asm!("lidt [eax]", in("eax") &idtr);
	}
}

pub extern "x86-interrupt" fn unimplemented_interrupt() {
	panic!("unimplemented interrupt");
}
