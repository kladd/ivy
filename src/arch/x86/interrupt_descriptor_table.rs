use core::arch::asm;

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
pub struct InterruptStackFrame {
	irq: u32,
	error_code: u32,
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

#[macro_export]
macro_rules! isr {
	($irq:expr, $name:ident) => {{
		#[naked]
		unsafe extern "C" fn handle_irq() -> ! {
			::core::arch::asm!(
				concat!( "cli; push 0; push ", stringify!($irq)),
				r#"
				cli
				pushad

				mov eax, esp
				add eax, 32
				push eax

				call {}

				add esp, 4
				popad
				add esp, 8
				sti
				iretd
				"#,
				sym $name,
				options(noreturn)
			)
		}
		($irq as usize, handle_irq as u32)
	}}
}

#[macro_export]
macro_rules! isr_code {
	($irq:expr, $name:ident) => {{
		#[naked]
		unsafe extern "C" fn handle_irq() -> ! {
			::core::arch::asm!(
				concat!( "cli; push ", stringify!($irq)),
				r#"
				cli
				pushad

				mov eax, esp
				add eax, 32
				push eax

				call {}

				add esp, 4
				popad
				add esp, 4
				sti
				iretd
				"#,
				sym $name,
				options(noreturn)
			)
		}
		($irq as usize, handle_irq as u32)
	}};
}

pub fn init_idt() {
	unsafe {
		// Table 6-1. Protected-Mode Exceptions and Interrupts
		register_handler(isr!(0, print_irq));
		register_handler(isr!(1, print_irq));
		register_handler(isr!(2, print_irq));
		register_handler(isr!(3, print_irq));
		register_handler(isr!(4, print_irq));
		register_handler(isr!(5, print_irq));
		register_handler(isr!(6, print_irq));
		register_handler(isr!(7, print_irq));

		register_handler(isr_code!(8, print_irq));

		register_handler(isr!(9, print_irq));

		register_handler(isr_code!(10, print_irq));
		register_handler(isr_code!(11, print_irq));
		register_handler(isr_code!(12, print_irq));
		register_handler(isr_code!(13, print_irq));
		register_handler(isr_code!(14, print_irq));

		register_handler(isr!(16, print_irq));

		register_handler(isr_code!(17, print_irq));

		register_handler(isr!(18, print_irq));
		register_handler(isr!(19, print_irq));
		register_handler(isr!(20, print_irq));

		flush_idt();
	}
}

pub fn register_handler(handler: (usize, u32)) {
	unsafe {
		DESCRIPTOR_TABLE[handler.0] =
			InterruptDescriptor::new(handler.1, 0x08, 0x8E);
	}
}

unsafe fn flush_idt() {
	let idtr = DescriptorTableRegister::new(&DESCRIPTOR_TABLE);
	asm!("lidt [eax]", in("eax") &idtr);
}

#[no_mangle]
extern "C" fn print_irq(es: &InterruptStackFrame) {
	panic!("{:?}", es);
}
