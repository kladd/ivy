use core::{
	arch::asm,
	fmt::{Debug, Formatter},
};

#[repr(C, packed)]
struct SegmentDescriptor {
	lim0_15: u16,
	base0_15: u16,
	base16_23: u8,
	access: u8,
	lim16_19_gavl: u8,
	base24_31: u8,
}

#[repr(C, packed)]
pub struct TSSDescriptor {
	low: SegmentDescriptor,
	base32_64: u32,
	reserved: u32,
}

#[repr(C, packed)]
pub struct FixedGDT {
	kernel_nl: SegmentDescriptor,
	kernel_cs: SegmentDescriptor,
	kernel_ss: SegmentDescriptor,
	user_nl: SegmentDescriptor,
	user_ss: SegmentDescriptor,
	user_cs: SegmentDescriptor,
	tss: TSSDescriptor,
}

extern "C" {
	fn boot_gdt();
	fn boot_tss();
}

pub fn adopt_boot_gdt() -> &'static mut FixedGDT {
	let gdt = unsafe { &mut *(boot_gdt as *mut FixedGDT) };
	gdt.tss.low.base0_15 = boot_tss as u16;
	gdt.tss.low.base16_23 = (boot_tss as u64 >> 16) as u8;
	gdt.tss.low.base24_31 = (boot_tss as u64 >> 24) as u8;
	gdt.tss.base32_64 = (boot_tss as u64 >> 32) as u32;
	gdt.tss.low.lim0_15 = 103;
	gdt.tss.low.access = 0xE9;

	unsafe { asm!("ltr ax", in("ax") 0x30) };

	gdt
}

impl Debug for FixedGDT {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		writeln!(f, "0x00 {:?}", self.kernel_nl)?;
		writeln!(f, "0x08 {:?}", self.kernel_cs)?;
		writeln!(f, "0x10 {:?}", self.kernel_ss)?;
		writeln!(f, "0x18 {:?}", self.user_nl)?;
		writeln!(f, "0x20 {:?}", self.user_ss)?;
		writeln!(f, "0x28 {:?}", self.user_cs)?;
		writeln!(f, "0x30 {:?}", self.tss)
	}
}

impl Debug for TSSDescriptor {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		let low = &self.low;
		let scratch = self.base32_64;
		writeln!(f, "{low:?}")?;
		writeln!(f, "    base 32:64 = {scratch:08X}")?;
		let scratch = self.reserved;
		writeln!(f, "    reserved   = {scratch:08X}")
	}
}

impl Debug for SegmentDescriptor {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		let mut scratchw;
		let mut scratchb;
		scratchw = self.lim0_15;
		writeln!(f, "")?;
		writeln!(f, "    lim  00:15 = {scratchw:04X}")?;
		scratchw = self.base0_15;
		writeln!(f, "    base 00:15 = {scratchw:04X}")?;
		scratchb = self.base16_23;
		writeln!(f, "    base 16:23 = {scratchb:02X}")?;
		scratchb = self.access;
		writeln!(f, "        access = {scratchb:02X}")?;
		scratchb = self.lim16_19_gavl;
		writeln!(f, "    lim 16:19* = {scratchb:02X}")?;
		scratchb = self.base24_31;
		writeln!(f, "    base 24:31 = {scratchb:02X}")
	}
}
