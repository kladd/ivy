#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(naked_functions)]
// TODO: Un-suppress these warnings.
#![allow(dead_code)]
#![allow(unused_variables)]

extern crate alloc;

mod arch;
#[macro_use]
mod debug;
mod devices;
mod kalloc;
mod logger;
mod mem;
mod multiboot;
mod proc;
mod sync;

use alloc::vec;
use core::{arch::asm, panic::PanicInfo, ptr, sync::atomic::Ordering};

use log::{debug, error, info};

use crate::{
	arch::amd64::{
		cli,
		clock::init_clock,
		hlt,
		idt::init_idt,
		pic::init_pic,
		sti,
		vmem::{PageTable, BOOT_PML4_TABLE},
	},
	devices::{serial::init_serial, video::Video},
	kalloc::KernelAllocator,
	logger::KernelLogger,
	mem::{frame::FrameAllocator, page::Page, KERNEL_BASE},
	multiboot::{MultibootInfo, MultibootModuleEntry},
	proc::Task,
};

static LOGGER: KernelLogger = KernelLogger;

const USER_PROGRAM: &[u8] = &[
	0x50, 0x49, 0xb8, 0xef, 0xbe, 0xad, 0xde, 0x00, 0x00, 0x00, 0x00, 0x0f,
	0x05, 0x58, 0xc3, 0x90,
];

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	cli();
	error!("kernel {info:#}");
	hlt();
	unreachable!();
}

#[no_mangle]
pub extern "C" fn kernel_start(
	multiboot_magic: u32,
	multiboot_info: &MultibootInfo,
) -> ! {
	init_serial();

	log::set_logger(&LOGGER).unwrap();
	log::set_max_level(log::STATIC_MAX_LEVEL);

	debug!("{:#08X?}", multiboot_magic);
	kdbg!(multiboot_info);

	FrameAllocator::init(0x40000000, 0x200000 * 40);
	KernelAllocator::init(_kernel_end as usize, 0x200000);

	init_idt();
	init_pic();
	init_clock();

	// sti();

	// let mut screen = Video::new(0x800000);
	// screen.test();
	info!("kernel_end: 0x{:016X}", _kernel_end as usize);

	let task = Task::new("gp_fault", say_hello);
	// Switch to task page directory.
	unsafe { asm!("mov cr3, {}", in(reg) task.cr3) };
	// load task code.
	unsafe {
		ptr::copy_nonoverlapping(
			USER_PROGRAM.as_ptr(),
			0x400000 as *mut u8,
			USER_PROGRAM.len(),
		);
	}
	// sysret to user program.
	unsafe {
		asm!(
			r#"
	cli
	mov rsp, r11
	mov r11, 0x002
	sysretq
	"#, in("r11") 0x404000, in("rcx") 0x400000
		)
	}
	// TODO: Why the hell does this stack need to be so large?
	// also TODO: task.rsp should be allocated in not kernel memory.
	// also TODO: r11 above (rflags) should enable interrupts, but returning
	//            from handlers is busted.

	loop {
		hlt()
	}
}

enum PageTableLevel {
	PML4,
	PDP,
	PD,
}

impl PageTableLevel {
	pub fn indent(&self) -> &'static str {
		match self {
			Self::PML4 => "",
			Self::PDP => "    ",
			Self::PD => "        ",
		}
	}

	pub fn next(
		&self,
		entry: usize,
	) -> Option<(*mut PageTable, PageTableLevel)> {
		match self {
			Self::PML4 => Some((
				((entry & !0xFFusize) + KERNEL_BASE) as *mut PageTable,
				PageTableLevel::PDP,
			)),
			Self::PDP => Some((
				((entry & !0xFFusize) + KERNEL_BASE) as *mut PageTable,
				PageTableLevel::PD,
			)),
			Self::PD => None,
		}
	}
}

fn dump_pt(pt: *mut PageTable, level: PageTableLevel) {
	for (i, ent) in unsafe { &*pt }.0.iter().enumerate() {
		if *ent != 0 {
			info!("{}[{i:03}] = {:016X}", level.indent(), ent & !0b10000);
			level.next(*ent).map(|(pt, pl)| {
				info!("{}[ ~ ] = {:016X}", level.indent(), pt as usize);
				dump_pt(pt, pl)
			});
		}
	}
}

#[no_mangle]
#[naked]
pub unsafe extern "C" fn handle_syscall() {
	asm!("mov r12, 0xdecafbad; ud2", options(noreturn));
}

// Assembled as USER_PROGRAM, this does nothing but document what that array
// means.
fn say_hello() {
	unsafe { asm!("mov r8, 0xdeadbeef; syscall") };
}

extern "C" {
	fn _kernel_end();
}
