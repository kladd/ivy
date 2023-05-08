use alloc::{format, string::String, vec, vec::Vec};
use core::{
	fmt::{Display, Formatter, Write},
	mem,
};

use crate::{
	fs::inode::{Inode, InodeHash},
	std::io::{SerialTerminal, VideoTerminal},
};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum DeviceInode {
	Root,
	Console,
	Serial,
}

impl DeviceInode {
	pub fn lookup(&self, name: &str) -> Option<Inode> {
		match name {
			"CONSOLE" => Some(Inode::Device(DeviceInode::Console)),
			"SERIAL" => Some(Inode::Device(DeviceInode::Serial)),
			_ => None,
		}
	}

	pub fn readdir(&self) -> Vec<Inode> {
		match self {
			DeviceInode::Root => {
				vec![DeviceInode::Console.into(), DeviceInode::Serial.into()]
			}
			DeviceInode::Console => vec![DeviceInode::Console.into()],
			DeviceInode::Serial => vec![DeviceInode::Serial.into()],
		}
	}

	pub fn hash(&self) -> InodeHash {
		InodeHash::Device(mem::discriminant(self))
	}
}

// TODO: This not here.
impl DeviceInode {
	pub fn read_line(&self) -> String {
		match self {
			DeviceInode::Console => VideoTerminal::global_mut().read_line(),
			DeviceInode::Serial => SerialTerminal::global_mut().read_line(),
			_ => unimplemented!(),
		}
	}
}

// TODO: Again, all this probably comes from `stat` when it exists.
impl DeviceInode {
	pub fn name(&self) -> String {
		match self {
			Self::Root => String::from("/"),
			node => format!("{node}").to_ascii_uppercase(),
		}
	}

	pub fn size(&self) -> usize {
		0
	}

	pub fn is_dir(&self) -> bool {
		match self {
			Self::Root => true,
			_ => false,
		}
	}
}

impl Write for DeviceInode {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		match self {
			DeviceInode::Console => VideoTerminal::global_mut().write_str(s),
			DeviceInode::Serial => SerialTerminal::global_mut().write_str(s),
			_ => unimplemented!(),
		}
	}
}

impl From<DeviceInode> for Inode {
	fn from(value: DeviceInode) -> Self {
		Self::Device(value)
	}
}

impl Display for DeviceInode {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "{:?}", self)
	}
}
