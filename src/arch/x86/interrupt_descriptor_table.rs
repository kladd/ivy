use core::{arch::asm, fmt::Write};

use crate::arch::x86::descriptor_table::DescriptorTableRegister;

const MAX_INTERRUPTS: usize = 256;

static mut DESCRIPTOR_TABLE: [InterruptDescriptor; MAX_INTERRUPTS] =
	[InterruptDescriptor::null(); MAX_INTERRUPTS];

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct InterruptDescriptor {
	isr_low: u16,
	kernel_cs: u16,
	_zero: u8,
	flags: u8,
	isr_high: u16,
}

#[derive(Debug)]
#[repr(C)]
struct ExceptionStackFrame {
	eip: u32,
	cs: u32,
	eflags: u32,
	esp: u32,
	ss: u32,
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

pub fn init_idt() {
	unsafe {
		for i in 0..32 {
			DESCRIPTOR_TABLE[i] = InterruptDescriptor::new(
				unimplemented_interrupt_handler as u32,
				0x08,
				0x8E,
			);
		}
		flush_idt();
	}
}

unsafe fn flush_idt() {
	let idtr = DescriptorTableRegister::new(&DESCRIPTOR_TABLE);
	asm!("lidt [eax]", in("eax") &idtr);
}

#[no_mangle]
extern "C" fn print_exception_stack_frame(es: &ExceptionStackFrame) {
	kdbg!(es);
}

extern "C" {
	// main.asm
	fn unimplemented_interrupt_handler() -> !;
}
