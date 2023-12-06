use alloc::vec::Vec;

use log::info;

use crate::arch::amd64::{inb, inl, inw, outl};

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

const PCI_ENABLE: u32 = 0x80000000;

const PCI_VENDOR_ID: u8 = 0x00;
const PCI_HEADER_TYPE: u8 = 0x0E;
const PCI_PROG: u8 = 0x09;
const PCI_REVISION_ID: u8 = 0x08;
const PCI_CLASS: u8 = 0x0A;
const PCI_ABAR: u8 = 0x24;

fn pci_address(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
	// 31      enabled
	// 30 - 24 reserved
	// 23 - 16 bus
	// 15 - 11 device
	// 10 - 08 function
	// 07 - 00 register offset
	((bus as u32) << 16)
		| ((slot as u32) << 11)
		| ((func as u32) << 8)
		| offset as u32
		| PCI_ENABLE
}

fn read_pci_byte(bus: u8, slot: u8, func: u8, offset: u8) -> u8 {
	outl(CONFIG_ADDRESS, pci_address(bus, slot, func, offset));
	inb(CONFIG_DATA + ((offset as u16) & 3))
}

fn read_pci_word(bus: u8, slot: u8, func: u8, offset: u8) -> u16 {
	outl(CONFIG_ADDRESS, pci_address(bus, slot, func, offset));
	inw(CONFIG_DATA + ((offset as u16) & 2))
}

fn read_pci_dword(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
	outl(CONFIG_ADDRESS, pci_address(bus, slot, func, offset));
	inl(CONFIG_DATA)
}

fn write_pci_dword(bus: u8, slot: u8, func: u8, offset: u8, val: u32) {
	outl(CONFIG_ADDRESS, pci_address(bus, slot, func, offset));
	outl(CONFIG_DATA, val);
}

fn dump_device(bus: u8, slot: u8, func: u8) {
	let class = read_pci_word(bus, slot, func, PCI_CLASS);
	let vendor = read_pci_word(bus, slot, func, PCI_VENDOR_ID);
	info!(
		"pci {bus:02X}:{slot:02X}.{func} [{class:04X}] {}",
		vendor_display(vendor)
	);
}

fn vendor_display(vendor: u16) -> &'static str {
	match vendor {
		0x8086 => "Intel Corporation",
		0x1234 => "QEMU",
		_ => "Unknown",
	}
}

pub struct PCIDevice {
	bus: u8,
	slot: u8,
	func: u8,
}

impl PCIDevice {
	pub fn class(&self) -> u16 {
		read_pci_word(self.bus, self.slot, self.func, PCI_CLASS)
	}
}

pub fn enumerate_pci() -> Vec<PCIDevice> {
	let mut vec = Vec::new();

	for bus in 0..=u8::MAX {
		for slot in 0..32 {
			for func in 0..8 {
				let class = read_pci_dword(bus, slot, func, 0x00);
				if class == 0xFFFFFFFF {
					continue;
				}
				dump_device(bus, slot, func);
				vec.push(PCIDevice { bus, slot, func });

				if func == 0 {
					let header_type = read_pci_byte(bus, slot, func, 0x0E);
					if header_type & 0x80 == 0 {
						break;
					}
				}
			}
		}
	}

	vec
}
