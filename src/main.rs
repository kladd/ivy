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

use core::{arch::asm, panic::PanicInfo, ptr, sync::atomic::Ordering};

use log::{debug, error, info};

use crate::{
	arch::amd64::{
		cli, clock::init_clock, hlt, idt::init_idt, pic::init_pic, sti,
		vmem::PageTable,
	},
	devices::{serial::init_serial, video::Video},
	kalloc::KernelAllocator,
	logger::KernelLogger,
	mem::{frame::FrameAllocator, page::Page, PhysicalAddress, KERNEL_BASE},
	multiboot::{MultibootInfo, MultibootModuleEntry},
	proc::{Task, CPU},
};

static LOGGER: KernelLogger = KernelLogger;

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
	sti();

	// Load user program from initrd since we don't have a filesystem yet.
	let initrd_mod: &MultibootModuleEntry = unsafe {
		&*(PhysicalAddress(multiboot_info.mods_addr as usize).to_virtual())
	};
	kdbg!(initrd_mod);

	// let mut screen = Video::new(0x800000);
	// screen.test();
	info!("kernel_end: 0x{:016X}", _kernel_end as usize);

	// panic!();
	let mut cpu = CPU::default();
	cpu.store();

	// First user process.
	let task = Task::new("gp_fault");
	// Switch to task page directory.
	unsafe { asm!("mov cr3, {}", in(reg) task.cr3) };
	// Load task code into 4MB (arbitrarily chosen entry point).
	unsafe {
		ptr::copy_nonoverlapping(
			PhysicalAddress(initrd_mod.start as usize).to_virtual(),
			0x400000 as *mut u8,
			(initrd_mod.end - initrd_mod.start) as usize,
		);
	}
	// SYSRET to user program.
	unsafe {
		cpu.rsp3 = task.rsp;
		asm!("jmp _syscall_ret", in("rcx") task.rip, options(nostack, noreturn))
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
pub unsafe extern "C" fn syscall_enter() {
	asm!("mov r12, 0xdecafbad");
}

extern "C" {
	fn _kernel_end();
}
