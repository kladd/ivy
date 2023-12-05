use core::{
	arch::asm,
	fmt::{Debug, Formatter},
	mem::size_of,
};

use log::{debug, warn};

use crate::{
	arch::amd64::{cli, hlt, outb},
	kdbg,
};

const MAX_INTERRUPTS: usize = 256;

static mut DESCRIPTOR_TABLE: [InterruptEntry; MAX_INTERRUPTS] =
	[InterruptEntry::default(); MAX_INTERRUPTS];

#[derive(Debug)]
#[repr(packed)]
pub(super) struct IDTR {
	_limit: u16,
	_base: u64,
}

#[repr(C)]
pub struct Interrupt {
	rip: usize,
	cs: usize,
	rflags: usize,
	rsp: usize,
	ss: usize,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct InterruptEntry {
	isr_lo: u16,
	selector: u16,
	flags: u16,
	isr_mid: u16,
	isr_hi: u32,
	_rsvd: u32,
}

impl InterruptEntry {
	pub const fn default() -> Self {
		Self {
			isr_lo: 0,
			isr_mid: 0,
			isr_hi: 0,
			selector: 0,
			flags: 0,
			_rsvd: 0,
		}
	}

	pub const fn new(isr: usize, sel: u16, flags: u16) -> Self {
		Self {
			isr_lo: isr as u16,
			isr_mid: (isr >> 16) as u16,
			isr_hi: (isr >> 32) as u32,
			selector: sel,
			_rsvd: 0,
			flags,
		}
	}
}

impl Interrupt {
	pub fn eoi(isr: usize) {
		if isr > 40 {
			outb(0xA0, 0x20);
		}
		outb(0x20, 0x20);
	}
}

pub fn init_idt() {
	unsafe {
		debug!("{:016X?}", print_irq as usize);
		register_handler(1, print_irq);
		register_handler(2, print_irq);
		register_handler(3, breakpoint);
		register_handler(4, print_irq);
		register_handler(5, print_irq);
		register_handler(6, invalid_opcode);
		register_handler(7, print_irq);
		register_handler_code(8, print_irq_code);
		register_handler(9, print_irq);
		register_handler_code(10, print_irq_code);
		register_handler_code(11, print_irq_code);
		register_handler_code(12, print_irq_code);
		register_handler_code(13, print_irq_code);
		register_handler_code(14, page_fault);
		register_handler(16, print_irq);
		register_handler_code(17, print_irq_code);
		register_handler(18, print_irq);
		register_handler(19, print_irq);
		register_handler(20, print_irq);
		register_handler_code(21, print_irq_code);
		register_handler(28, print_irq);
		register_handler_code(29, print_irq_code);
		register_handler_code(30, print_irq_code);

		flush_idt();
	}
}

impl IDTR {
	pub fn new<T, const N: usize>(table: &[T; N]) -> Self {
		Self {
			_limit: size_of::<[T; N]>() as u16 - 1,
			_base: table as *const [T; N] as u64,
		}
	}
}

extern "x86-interrupt" fn print_irq(interrupt: Interrupt) {
	panic!("{interrupt:#?}");
}

extern "x86-interrupt" fn invalid_opcode(interrupt: Interrupt) {
	panic!("#UD({:016X}): {interrupt:#?}", interrupt.rip);
}

extern "x86-interrupt" fn page_fault(interrupt: Interrupt, error: usize) {
	panic!(
		"#PF({:016X}, error: {error:016X}): {interrupt:#?}",
		interrupt.rip
	);
}

extern "x86-interrupt" fn breakpoint(interrupt: Interrupt) {
	warn!("#BP({:016X}): {interrupt:#?}", interrupt.rip);
	cli();
	hlt();
}

extern "x86-interrupt" fn print_irq_code(interrupt: Interrupt, error: usize) {
	panic!("{interrupt:#?} {error:#?}");
}

pub fn register_handler(
	irq: usize,
	handler: extern "x86-interrupt" fn(Interrupt),
) {
	// TODO: dedupe.
	unsafe {
		let flags = if irq < 14 { 0x8F } else { 0x8E };
		DESCRIPTOR_TABLE[irq] =
			// TODO: left shift by 8 is a hack porting from x86.
			InterruptEntry::new(handler as usize, 0x08, flags << 8);
	}
}

pub fn register_handler_code(
	irq: usize,
	handler: extern "x86-interrupt" fn(Interrupt, usize),
) {
	// TODO: dedupe.
	unsafe {
		let flags = if irq < 14 { 0x8F } else { 0x8E };
		DESCRIPTOR_TABLE[irq] =
			// TODO: left shift by 8 is a hack porting from x86.
			InterruptEntry::new(handler as usize, 0x08, flags << 8);
	}
}

unsafe fn flush_idt() {
	let idtr = kdbg!(IDTR::new(&DESCRIPTOR_TABLE));
	asm!("lidt [rax]", in("rax") &idtr);
}

impl Debug for Interrupt {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("Interrupt")
			.field("rip", &format_args!("0x{:016x}", self.rip))
			.field("cs", &format_args!("0x{:04x}", self.cs))
			.field("rflags", &format_args!("0x{:016x}", self.rflags))
			.field("rsp", &format_args!("0x{:016x}", self.rsp))
			.field("ss", &format_args!("0x{:04x}", self.ss))
			.finish()
	}
}
