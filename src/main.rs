#![no_std]
#![no_main]

#[macro_use]
mod debug;
mod arch;
mod boot;
mod serial;
mod vga;
mod x86;

use core::{arch::asm, fmt::Write, panic::PanicInfo};

use x86::descriptor_table::{
	gdt, gdt::SegmentDescriptor, idt, idt::InterruptDescriptor,
};

use crate::{
	boot::{MultibootInfo, MULTIBOOT_MAGIC},
	serial::COM1,
	vga::VGA,
	x86::{common::halt, descriptor_table::DescriptorTableRegister, pic},
};

#[panic_handler]
unsafe fn panic(_info: &PanicInfo) -> ! {
	kprintf!("kernel {}", _info);
	halt()
}

fn exception_handler(stack_0: u32) -> ! {
	panic!("CPU exception {}", stack_0);
}

fn unimplemented_irq() {
	kprintf!("unimplemented interrupt");
	unsafe {
		asm!("iret");
	}
}

#[no_mangle]
pub extern "C" fn kernel_start(
	multiboot_magic: u32,
	multiboot_info: &MultibootInfo,
) -> ! {
	assert_eq!(multiboot_magic, MULTIBOOT_MAGIC);
	kdbg!(multiboot_info);

	let gdt = [
		SegmentDescriptor::null(),
		SegmentDescriptor::new(0xFFFFFFFF, 0, 0x9A, 0xCF),
		SegmentDescriptor::new(0xFFFFFFFF, 0, 0x92, 0xCF),
		SegmentDescriptor::new(0xFFFFFFFF, 0, 0xFA, 0xCF),
		SegmentDescriptor::new(0xFFFFFFFF, 0, 0xF2, 0xCF),
	];
	let gdtr = DescriptorTableRegister::new(gdt);
	unsafe {
		gdt::flush(&gdtr);
	}

	let mut idt = [InterruptDescriptor::null(); 256];
	// Exceptions
	for i in 0..32 {
		idt[i] = InterruptDescriptor::new(exception_handler as u32, 0x08, 0x8E);
	}
	// Remapped PIC
	for i in 32..48 {
		idt[i] = InterruptDescriptor::new(unimplemented_irq as u32, 0x08, 0x8E);
	}
	let idtr = DescriptorTableRegister::new(idt);
	unsafe {
		idt::flush(&idtr);
		asm!("sti");
	}

	pic::init();

	COM1.init();
	kprintf!("If you can read this, {} logging works", "debug");

	VGA.clear_screen();
	VGA.disable_cursor();
	writeln!(VGA, "Welcome to Ivy OS!").unwrap();

	halt()
}
