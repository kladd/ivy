pub mod frame;
pub mod page;

pub const PAGE_SIZE: usize = 0x200000;

#[derive(Debug)]
pub struct VirtualAddress(usize);

#[derive(Debug)]
pub struct PhysicalAddress(pub usize);

pub const KERNEL_BASE: usize = 0xFFFFFFFF80000000;
