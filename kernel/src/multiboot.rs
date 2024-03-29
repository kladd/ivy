use crate::mem::PAGE_SIZE;

const MEM_INFO_FLAG: u32 = 0x1;

#[derive(Debug)]
#[repr(packed)]
pub struct MultibootInfo {
	pub flags: u32,
	pub mem_lower: u32,
	pub mem_upper: u32,
	pub boot_device: u32,
	pub cmdline: u32,
	pub mods_count: u32,
	pub mods_addr: u32,
	pub elf_num: u32,
	pub elf_size: u32,
	pub elf_addr: u32,
	pub elf_shndx: u32,
	pub mmap_length: u32,
	pub mmap_addr: u32,
	pub drives_length: u32,
	pub drives_addr: u32,
	pub config_table: u32,
	pub boot_loader_name: u32,
	pub apm_table: u32,
	pub vbe_control_info: u32,
	pub vbe_mode_info: u32,
	pub vbe_mode: u16,
	pub vbe_interface_seg: u16,
	pub vbe_interface_off: u16,
	pub vbe_interface_len: u16,
	pub framebuffer_addr: u64,
	pub framebuffer_pitch: u32,
	pub framebuffer_width: u32,
	pub framebuffer_height: u32,
	pub framebuffer_bpp: u8,
	pub framebuffer_type: u8,
	// pub framebuffer_palette_addr: u32,
	// pub framebuffer_palette_num_colors: u16,
	pub framebuffer_red_field_position: u8,
	pub framebuffer_red_mask_size: u8,
	pub framebuffer_green_field_position: u8,
	pub framebuffer_green_mask_size: u8,
	pub framebuffer_blue_field_position: u8,
	pub framebuffer_blue_mask_size: u8,
}

#[repr(C)]
#[derive(Debug)]
pub struct MultibootModuleEntry {
	pub start: u32,
	pub end: u32,
	pub string: u32,
	reserved: u32,
}

#[repr(packed)]
#[derive(Debug)]
pub struct MultibootMmapEntry {
	pub size: u32,
	pub addr: u64,
	pub len: u64,
	pub kind: u32,
}

impl MultibootMmapEntry {
	pub fn pages(&self) -> usize {
		(self.len as usize + PAGE_SIZE - 1) / PAGE_SIZE
	}
}
