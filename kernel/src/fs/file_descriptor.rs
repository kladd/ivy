use crate::fs::inode::Inode;

#[derive(Debug)]
pub struct FileDescriptor {
	pub(super) offset: usize,
	pub inode: Inode,
}

impl FileDescriptor {
	pub fn new(inode: Inode) -> Self {
		Self { offset: 0, inode }
	}
}
