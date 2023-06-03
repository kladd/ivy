pub mod frame;
pub mod page;

pub const PAGE_SIZE: usize = 0x200000;

#[derive(Debug)]
pub struct VirtualAddress(usize);

#[derive(Debug)]
pub struct PhysicalAddress(pub usize);

impl PhysicalAddress {
	pub fn to_virtual<T>(&self) -> *mut T {
		(self.0 + KERNEL_BASE) as *mut T
	}
}

pub const KERNEL_BASE: usize = 0xFFFFFFFF80000000;
