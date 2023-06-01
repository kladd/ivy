use crate::mem::{PhysicalAddress, PAGE_SIZE};

pub struct Page(usize);

const MASK: usize = !(PAGE_SIZE - 1);

impl Page {
	pub fn new(
		frame: PhysicalAddress,
		present: bool,
		rw: bool,
		user: bool,
	) -> Self {
		let PhysicalAddress(pfn) = frame;

		Self(
			(pfn & MASK)
				| (present as usize)
				| ((rw as usize) << 1)
				| ((user as usize) << 2)
				| 0x80, // TODO: not huge
		)
	}

	pub fn entry(&self) -> usize {
		self.0
	}
}
