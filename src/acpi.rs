//! https://uefi.org/htmlspecs/ACPI_Spec_6_4_html/21_ACPI_Data_Tables_and_Table_Def_Language/ACPI_Data_Tables.html

use alloc::{
	string::{String, ToString},
	vec::Vec,
};
use core::{mem, ptr, slice};

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
#[derive(Debug)]
pub struct ACPITableHeader {
	pub signature: [u8; 4],
	length: u32,
	revision: u8,
	checksum: u8,
	oem_id: [u8; 6],
	oem_table_id: [u8; 8],
	oem_revision: u32,
	asl_compiler_id: [u8; 4],
	asl_compiler_revision: u32,
}

#[derive(Debug)]
pub struct RSDT {
	header: ACPITableHeader,
	entries: Vec<u32>,
}

impl RSDP {
	pub fn rsdt(&self) -> RSDT {
		let header: ACPITableHeader = unsafe {
			core::intrinsics::unaligned_volatile_load(
				PhysicalAddress(self.rsdt_addr as usize).to_virtual(),
			)
		};

		let header_offset = mem::size_of::<ACPITableHeader>();
		let entries_count = (header.length as usize - header_offset) / 4;
		let mut entries = Vec::with_capacity(entries_count);

		for i in 0..entries_count {
			let addr = unsafe {
				core::intrinsics::unaligned_volatile_load(
					PhysicalAddress(
						self.rsdt_addr as usize
							+ header_offset + (i * mem::size_of::<u32>()),
					)
					.to_virtual(),
				)
			};
			entries.push(addr);
		}

		RSDT { header, entries }
	}
}

impl RSDT {
	pub fn test(&self) {
		for a in &self.entries {
			let header: ACPITableHeader = unsafe {
				core::intrinsics::unaligned_volatile_load(
					PhysicalAddress(*a as usize).to_virtual(),
				)
			};
			debug!("{}", header.signature());
		}
	}
}

impl ACPITableHeader {
	pub fn signature(&self) -> &str {
		unsafe { core::str::from_utf8_unchecked(&self.signature[..]) }
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
