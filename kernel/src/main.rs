#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(naked_functions)]
#![feature(core_intrinsics)]
#![feature(pointer_byte_offsets)]
// TODO: Un-suppress these warnings.
#![allow(dead_code)]
#![allow(unused_variables)]

extern crate alloc;

mod arch;
#[macro_use]
mod debug;
mod devices;
mod elf;
mod font;
mod fs;
mod kalloc;
mod logger;
mod mem;
mod multiboot;
mod proc;
mod sync;
mod syscall;

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{
	arch::asm, cmp::min, fmt::Write, mem::size_of, panic::PanicInfo, ptr,
	slice, str,
};

use log::{debug, error};

use crate::{
	arch::amd64::{
		cli,
		clock::init_clock,
		gdt, hlt,
		idt::init_idt,
		pic::init_pic,
		sti, vmem,
		vmem::{map_physical_memory, PageTable, PML4},
	},
	devices::{
		ide, keyboard::init_keyboard, pci::enumerate_pci, serial, tty, vga,
	},
	fs::{device::DeviceFileSystem, ext2, fs0, inode::Inode},
	kalloc::KernelAllocator,
	logger::KernelLogger,
	mem::{
		frame, kernel_map, PhysicalAddress, KERNEL_LMA, KERNEL_VMA, PAGE_SIZE,
	},
	multiboot::{MultibootInfo, MultibootMmapEntry, MultibootModuleEntry},
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
) {
	gdt::adopt_boot_gdt();

	serial::init();
	log::set_logger(&LOGGER).unwrap();
	log::set_max_level(log::STATIC_MAX_LEVEL);

	debug!("{:#08X?}", multiboot_magic);
	kdbg!(multiboot_info);

	KernelAllocator::init(_kernel_end as usize, 0x200000);

	debug!("kernel end {:016X}", _kernel_end as usize - KERNEL_VMA);

	let kernel_page_table = PageTable::<PML4>::current_mut();
	let memory_map = read_memory_map(multiboot_info);
	identity_map_reserved(kernel_page_table, &memory_map);
	map_framebuffer(kernel_page_table, multiboot_info);

	init_idt();
	init_pic();
	init_clock();
	init_keyboard();

	vga::init();
	tty::init();

	map_physical_memory(512 * 2 * 0x200000);

	sti();

	init_frame_allocator(&memory_map);

	// Does nothing right now because in named cpu mode QEMU isn't populating
	// device class fields.
	let pci_devices = enumerate_pci();

	ide::init();

	let mods: &[MultibootModuleEntry] = unsafe {
		slice::from_raw_parts(
			PhysicalAddress(multiboot_info.mods_addr as usize).to_virtual(),
			multiboot_info.mods_count as usize,
		)
	};

	#[cfg(gfx)]
	init_gfx(multiboot_info, mods, kernel_page_table);

	// Map initrd to load the program stored there.
	kernel_map(
		kernel_page_table,
		PhysicalAddress(mods[0].start as usize),
		(mods[0].end as usize - mods[0].start as usize).div_ceil(PAGE_SIZE),
	);

	fs::init();
	let rootfs = Arc::new(ext2::FileSystem::new(0));
	fs0().mount_root(Inode::Ext2(rootfs.root()));
	fs0().mount("/dev", DeviceFileSystem.root());

	// First user process.
	let mut task = Task::new("user");

	// Switch to task page directory.
	unsafe { asm!("mov cr3, {}", in(reg) task.cr3) };
	elf::load(PhysicalAddress(mods[0].start as usize), &mut task);

	let mut cpu = CPU::new();
	let entry = task.rip;
	cpu.rsp3 = task.rsp;
	cpu.task = Box::into_raw(Box::new(task));
	cpu.store();

	// SYSRET to user program.
	unsafe {
		asm!("jmp _syscall_ret", in("rcx") entry, options(nostack, noreturn))
	}
}

#[cfg(gfx)]
fn init_gfx(
	multiboot_info: &MultibootInfo,
	mods: &[MultibootModuleEntry],
	kernel_page_table: &mut PML4,
) {
	kernel_map(
		kernel_page_table,
		PhysicalAddress(mods[0].start as usize),
		(mods[1].end as usize - mods[1].start as usize).div_ceil(PAGE_SIZE),
	);
	video::init(
		PhysicalAddress(multiboot_info.framebuffer_addr as usize),
		multiboot_info.framebuffer_width as usize
			* multiboot_info.framebuffer_height as usize,
		PhysicalAddress(mods[1].start as usize),
	);
	video_terminal::init();
}

fn read_memory_map(multiboot_info: &MultibootInfo) -> Vec<MultibootMmapEntry> {
	let mut map = Vec::new();

	let mmap_base: *const MultibootMmapEntry =
		PhysicalAddress(multiboot_info.mmap_addr as usize).to_virtual();
	let mmap_len =
		multiboot_info.mmap_length as usize / size_of::<MultibootMmapEntry>();

	let mut i = 0;
	loop {
		let region: MultibootMmapEntry =
			unsafe { ptr::read_unaligned(mmap_base.byte_offset(i)) };
		debug!("mmap[{i}] => {:#X?}, pages = {}", region, region.pages());
		i += region.size as isize + size_of::<u32>() as isize;

		map.push(region);

		if i >= multiboot_info.mmap_length as isize {
			break;
		}
	}

	map
}

fn identity_map_reserved(
	kernel_page_table: &mut PageTable<PML4>,
	memory_map: &Vec<MultibootMmapEntry>,
) {
	for region in memory_map {
		kernel_map(
			kernel_page_table,
			PhysicalAddress(region.addr as usize),
			min(1, region.pages()),
		);
	}
}

fn init_frame_allocator(memory_map: &Vec<MultibootMmapEntry>) {
	let largest_range = memory_map
		.iter()
		// Kind 1 = Available.
		.filter(|region| region.kind == 1)
		.max_by(|region_a, region_b| {
			// For alignment.
			let a_len = region_a.len;
			let b_len = region_b.len;
			a_len.cmp(&b_len)
		})
		.expect("No available memory regions.");
	// Alignment.
	let largest_range_addr = largest_range.addr;
	assert_eq!(
		KERNEL_LMA, largest_range.addr as usize,
		"Unexpected memory region: {}",
		largest_range_addr
	);
	assert!(
		largest_range.pages() > 512,
		"Frame allocator region too small."
	);

	// TODO: Don't hardcode frame allocator region.
	frame::init(0xA00000, 512 * PAGE_SIZE);
}

fn map_framebuffer(
	kernel_page_table: &mut PageTable<PML4>,
	multiboot_info: &MultibootInfo,
) {
	kernel_map(
		kernel_page_table,
		PhysicalAddress(multiboot_info.framebuffer_addr as usize),
		(multiboot_info.framebuffer_bpp as usize / u8::BITS as usize
			* multiboot_info.framebuffer_height as usize
			* multiboot_info.framebuffer_width as usize)
			.div_ceil(PAGE_SIZE),
	);
}

extern "C" {
	fn _kernel_end();
}
