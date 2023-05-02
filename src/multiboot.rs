use log::debug;

const MEM_INFO_FLAG: u32 = 0x1;

#[derive(Debug)]
#[repr(C)]
pub struct MultibootFlags(u32);

#[derive(Debug)]
#[repr(C)]
pub struct MultibootInfo {
	pub mem_lower: u32,
	pub mem_upper: u32,
}

impl MultibootInfo {
	pub fn read(flags: &MultibootFlags) -> Self {
		debug!("multiboot_flags = {:032b}", flags.0);
		assert_ne!(flags.0 & MEM_INFO_FLAG, 0);

		let base_ptr = flags as *const MultibootFlags as *const u32;

		Self {
			mem_lower: unsafe { *base_ptr.offset(1) },
			mem_upper: unsafe { *base_ptr.offset(2) },
		}
	}
}
