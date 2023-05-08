use core::arch::asm;

pub const PAGE_SIZE: u32 = 0x1000;

pub const PRESENT: u32 = 0x1;
pub const READ_WRITE: u32 = 0x2;

#[repr(align(0x1000))]
pub struct PageTable([u32; 1024]);

#[repr(align(0x1000))]
pub struct PageDirectory([u32; 512]);

impl PageTable {
	pub const fn new() -> Self {
		Self([0; 1024])
	}

	pub fn set(&mut self, phys: usize, virt: u32, flags: u32) {
		assert!(phys < self.0.len());
		*(unsafe { self.0.get_unchecked_mut(phys) }) = virt * PAGE_SIZE | flags;
	}
}

impl PageDirectory {
	pub const fn new() -> Self {
		Self([READ_WRITE; 512])
	}

	pub fn set(&mut self, i: usize, entry: &PageTable, flags: u32) {
		assert!(i < self.0.len());
		*(unsafe { self.0.get_unchecked_mut(i) }) =
			entry as *const _ as u32 | flags;
	}

	pub fn make_active(&self) {
		unsafe { asm!("mov cr3, {}", in(reg) self as *const _ as u32) };
	}
}

extern "C" {
	fn enable_paging();
}

static mut PAGE_DIRECTORY: PageDirectory = PageDirectory::new();
static mut PAGE_TABLE_1: PageTable = PageTable::new();
static mut PAGE_TABLE_2: PageTable = PageTable::new();

pub fn init_kernel_page_tables() {
	unsafe {
		// Identity map 0-8MB.
		for i in 0..1024 {
			PAGE_TABLE_1.set(i, i as u32, READ_WRITE | PRESENT);
			PAGE_TABLE_2.set(i, i as u32 + 1024, READ_WRITE | PRESENT);
		}
		PAGE_DIRECTORY.set(0, &PAGE_TABLE_1, READ_WRITE | PRESENT);
		PAGE_DIRECTORY.set(1, &PAGE_TABLE_2, READ_WRITE | PRESENT);
		PAGE_DIRECTORY.make_active();
		enable_paging();
	}
}
