use crate::mem::{PhysicalAddress, PAGE_SIZE};

pub struct Page(usize);

const MASK: usize = !(PAGE_SIZE - 1);
const PRESENT_BIT: usize = 0;

impl Page {
	pub fn new(frame: PhysicalAddress, flags: usize) -> Self {
		let PhysicalAddress(pfn) = frame;
		Self((pfn & MASK) | flags)
	}

	pub fn present(&self) -> bool {
		self.0 & (0x1 << PRESENT_BIT) != 0
	}

	pub fn from_entry(entry: usize) -> Option<Self> {
		if entry != 0 {
			Some(Self(entry))
		} else {
			None
		}
	}

	pub fn entry(&self) -> usize {
		self.0
	}
}
