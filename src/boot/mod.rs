pub const MULTIBOOT_MAGIC: u32 = 0x2BADB002;

#[derive(Debug)]
#[repr(C)]
pub struct MultibootInfo {
	flags: u32,
	mem_lower: u32,
	mem_upper: u32,
	// TODO: The rest.
}
