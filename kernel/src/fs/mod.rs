pub mod device;
pub mod ext2;
pub mod file_descriptor;
pub mod inode;

use alloc::{format, vec::Vec};

pub use file_descriptor::FileDescriptor;

use crate::{
	fs::inode::{Inode, InodeHash},
	sync::StaticPtr,
};

#[derive(Debug)]
pub struct MountPoint {
	host_inode_hash: Option<InodeHash>,
	guest_root_inode: Inode,
}

#[derive(Debug)]
pub struct FileSystem {
	mounts: Vec<MountPoint>,
}

impl FileSystem {
	pub fn root(&self) -> &Inode {
		&self
			.mounts
			.get(0)
			.expect("no root mounted")
			.guest_root_inode
	}

	pub fn mount_root(&mut self, root: Inode) {
		if !self.mounts.is_empty() {
			panic!("Root filesystem already mounted");
		}
		self.mount_inode(None, root);
	}

	pub fn mount(&mut self, path: &str, guest_root: Inode) {
		// TODO: Handle mount points that don't exist.
		// TODO: Mount with arbitrary relative paths instead of relative to
		//       `self.root()`.
		let host_inode = self
			.find(self.root(), path)
			.expect(&format!("Mount point doesn't exist: {path}"));
		self.mount_inode(Some(host_inode), guest_root);
	}

	pub fn find(&self, base: &Inode, path: &str) -> Option<Inode> {
		if path == "." {
			Some(base.clone())
		} else if path.starts_with("/") {
			self.find(self.root(), &path[1..])
		} else {
			let segments = path.split("/");
			let mut node = base.clone();

			for segment in segments {
				if segment.is_empty() {
					continue;
				}

				node = node.lookup(segment)?;

				// If this node is a mount point, start traversing that mounted
				// filesystem by returning its root here.
				if let Some(mp) = self.mount_point(&node) {
					node = mp
				}
			}

			Some(node)
		}
	}
}

impl FileSystem {
	fn mount_inode(&mut self, host_inode: Option<Inode>, guest_root: Inode) {
		self.mounts.push(MountPoint {
			host_inode_hash: host_inode.map(|inode| inode.hash()),
			guest_root_inode: guest_root,
		})
	}

	fn mount_point(&self, node: &Inode) -> Option<Inode> {
		for mp in &self.mounts {
			match mp.host_inode_hash {
				Some(ref hash) if *hash == node.hash() => {
					return Some(mp.guest_root_inode.clone())
				}
				_ => continue,
			}
		}
		None
	}
}

static FS: StaticPtr<FileSystem> = StaticPtr::new();

pub fn init() {
	FS.init(FileSystem {
		mounts: Vec::with_capacity(4),
	})
}

pub fn fs0() -> &'static mut FileSystem {
	FS.get()
}
