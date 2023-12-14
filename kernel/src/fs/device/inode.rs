use alloc::{format, string::String, vec, vec::Vec};
use core::{
	fmt::{Display, Formatter, Write},
	mem,
};

use crate::{
	devices::{serial::com1, tty::tty0},
	fs::inode::{Inode, InodeHash},
};

#[derive(Debug, Copy, Clone)]
pub enum DeviceInode {
	Root,
	Console,
	Serial,
}

impl DeviceInode {
	pub fn lookup(&self, name: &str) -> Option<Inode> {
		match name {
			"tty0" => Some(Inode::Device(DeviceInode::Console)),
			"com1" => Some(Inode::Device(DeviceInode::Serial)),
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
			DeviceInode::Console => tty0().lock().read_line(),
			DeviceInode::Serial => todo!(),
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
			DeviceInode::Console => tty0().lock().write_str(s),
			DeviceInode::Serial => com1().lock().write_str(s),
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
