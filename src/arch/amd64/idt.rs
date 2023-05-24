use core::{arch::asm, mem::size_of};

const MAX_INTERRUPTS: usize = 256;

static mut DESCRIPTOR_TABLE: [InterruptEntry; MAX_INTERRUPTS] =
	[InterruptEntry::default(); MAX_INTERRUPTS];

#[repr(packed)]
pub(super) struct IDTR {
	_limit: u16,
	_base: u64,
}

#[repr(C)]
#[derive(Debug)]
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

pub fn init_idt() {
	unsafe {
		register_handler(1, print_irq);
		register_handler(2, print_irq);
		register_handler(3, print_irq);
		register_handler(4, print_irq);
		register_handler(5, print_irq);
		register_handler(6, print_irq);
		register_handler(7, print_irq);
		register_handler_code(8, print_irq_code);
		register_handler(9, print_irq);
		register_handler_code(10, print_irq_code);
		register_handler_code(11, print_irq_code);
		register_handler_code(12, print_irq_code);
		register_handler_code(13, print_irq_code);
		register_handler_code(14, print_irq_code);
		register_handler(16, print_irq);
		register_handler_code(17, print_irq_code);
		register_handler_code(18, print_irq_code);
		register_handler_code(19, print_irq_code);
		register_handler_code(20, print_irq_code);

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
	let idtr = IDTR::new(&DESCRIPTOR_TABLE);
	asm!("lidt [rax]", in("rax") &idtr);
}
