use alloc::string::String;
use core::{ptr, slice};

use log::{debug, trace};

use crate::mem::PhysicalAddress;

#[repr(C)]
#[derive(Debug)]
pub struct RSDP {
	signature: [u8; 8],
	checksum: u8,
	pub oem_id: [u8; 6],
	rev: u8,
	pub rsdt_addr: u32,
	len: u32,
	xsdt_addr: u64,
	echecksum: u8,
	reserved: [u8; 3],
}

#[repr(C)]
pub struct RSDT {
	pub signature: [u8; 4],
	len: u32,
}

impl RSDP {
	pub fn rsdt(&self) {
		let x: [u8; 4] = unsafe {
			core::intrinsics::unaligned_volatile_load(
				PhysicalAddress(self.rsdt_addr as usize).to_virtual(),
			)
		};
		debug!("{}", String::from_utf8_lossy(&x[..]));
	}
}

// https://uefi.org/specs/ACPI/6.5/05_ACPI_Software_Programming_Model.html#finding-the-rsdp-on-ia-pc-systems
pub fn find_rsdp() -> *const RSDP {
	let edba: *mut u8 = PhysicalAddress(0xe0000).to_virtual();
	//                 128kb.
	for i in (0..0x1f400).step_by(16) {
		let s = unsafe { slice::from_raw_parts(edba.offset(i), 16) };
		if s.starts_with(b"RSD PTR ") {
			trace!("[{:016X}]: RSD PTR", s.as_ptr() as u64);
			return unsafe { edba.offset(i) as *mut RSDP };
		}
	}
	panic!("Could not locate RSDP");
}
