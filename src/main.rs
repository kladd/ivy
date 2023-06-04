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
mod font;
mod kalloc;
mod logger;
mod mem;
mod multiboot;
mod proc;
mod sync;

use core::{arch::asm, fmt::Write, panic::PanicInfo, ptr, slice};

use log::{debug, error, info};

use crate::{
	arch::amd64::{
		cli, clock::init_clock, hlt, idt::init_idt, pic::init_pic, sti,
		vmem::PageTable,
	},
	devices::{
		keyboard::{init_keyboard, KBD},
		serial::init_serial,
		video::Video,
		video_terminal::VideoTerminal,
	},
	font::PSF2Font,
	kalloc::KernelAllocator,
	logger::KernelLogger,
	mem::{frame::FrameAllocator, PhysicalAddress, KERNEL_BASE},
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
	init_keyboard();
	sti();

	// Load user program from initrd since we don't have a filesystem yet.
	let mods: &[MultibootModuleEntry] = unsafe {
		slice::from_raw_parts(
			PhysicalAddress(multiboot_info.mods_addr as usize).to_virtual(),
			multiboot_info.mods_count as usize,
		)
	};

	let font: &PSF2Font =
		unsafe { &*PhysicalAddress(mods[1].start as usize).to_virtual() };
	font.debug('&');

	let screen = Video::new(PhysicalAddress(0x800000).to_virtual(), font);
	let mut video_term = unsafe { VideoTerminal::new(screen, &mut KBD) };

	video_term.clear();
	loop {
		match video_term.read_line().as_str() {
			"test" => video_term.test(),
			"echo" => video_term.write_str("this is an echo\n").unwrap(),
			_ => {}
		}
	}

	let mut cpu = CPU::default();
	cpu.store();

	// First user process.
	let task = Task::new("gp_fault");
	// Switch to task page directory.
	unsafe { asm!("mov cr3, {}", in(reg) task.cr3) };
	// Load task code into 4MB (arbitrarily chosen entry point).
	unsafe {
		ptr::copy_nonoverlapping(
			PhysicalAddress(mods[0].start as usize).to_virtual(),
			0x400000 as *mut u8,
			(mods[0].end - mods[0].start) as usize,
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
