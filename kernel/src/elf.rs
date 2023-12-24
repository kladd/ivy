use core::{ptr, slice};

use log::{debug, info};

use crate::{mem::PhysicalAddress, proc::Task};

const EI_NIDENT: usize = 16;

#[repr(u32)]
#[derive(Debug)]
enum ProgramHeaderType {
	Null,
	Load,
	Dynamic,
	Interp,
	Note,
	ShLib,
	PHDR,
	LoProc = 0x70000000,
	HiProc = 0x7FFFFFFF,
}

#[repr(u16)]
#[derive(Debug)]
enum ExecutableType {
	None,
	Rel,
	Exec,
	Dyn,
	Core,
	LoProc = 0xFF00,
	HiProc = 0xFFFF,
}

#[repr(u32)]
enum ProgramHeaderFlags {
	X = 0x1,
	W = 0x2,
	R = 0x4,
}

#[repr(C)]
#[derive(Debug)]
struct ELF64Header {
	e_ident: [u8; EI_NIDENT],
	// TODO: No panics on malformed ELF headers.
	e_type: ExecutableType,
	e_machine: u16,
	e_version: u32,
	e_entry: u64,
	e_phoff: u64,
	e_shoff: u64,
	e_flags: u32,
	e_ehsize: u16,
	e_phentsize: u16,
	e_phnum: u16,
	e_shentsize: u16,
	e_shnum: u16,
	e_shstrndx: u16,
}

#[repr(C)]
#[derive(Debug)]
struct ELF64ProgramHeader {
	// TODO: No panics on malformed ELF headers.
	p_type: ProgramHeaderType,
	p_flags: u32,
	p_offset: u64,
	p_vaddr: u64,
	p_paddr: u64,
	p_filesz: u64,
	p_memsz: u64,
	p_align: u64,
}

pub fn load(elf: PhysicalAddress, task: &mut Task) {
	let header: &ELF64Header =
		unsafe { &*(elf.to_virtual() as *const ELF64Header) };
	debug!("{header:#?}");
	let program_headers: &[ELF64ProgramHeader] = unsafe {
		slice::from_raw_parts(
			elf.offset(header.e_phoff as usize).to_virtual(),
			header.e_phnum as usize,
		)
	};

	for phdr in program_headers
		.iter()
		.filter(|phdr| matches!(phdr.p_type, ProgramHeaderType::Load))
	{
		debug!("{phdr:#X?}");
		unsafe {
			ptr::copy_nonoverlapping(
				elf.offset(phdr.p_offset as usize).to_virtual(),
				phdr.p_vaddr as usize as *mut u8,
				phdr.p_filesz as usize,
			)
		}
	}

	task.register_state.rip = header.e_entry;
}
