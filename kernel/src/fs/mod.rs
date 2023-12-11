use core::mem::MaybeUninit;

use crate::{
	fs::{
		file_descriptor::FileDescriptor,
		inode::{Inode, InodeHash},
	},
	sync::StaticPtr,
};

pub mod device;
pub mod ext2;
pub mod file_descriptor;
pub mod inode;

#[derive(Debug)]
pub struct MountPoint {
	host_inode_hash: Option<InodeHash>,
	guest_root_inode: Inode,
}

#[derive(Debug)]
pub struct FileSystem {
	n_mounts: usize,
	mounts: [MaybeUninit<MountPoint>; 4],
}

impl FileSystem {
	pub fn root(&self) -> &Inode {
		unsafe { &self.mounts[0].assume_init_ref().guest_root_inode }
	}

	// TODO: Result, not option.
	pub fn open(&self, base: &Inode, path: &str) -> Option<FileDescriptor> {
		self.find(base, path).map(FileDescriptor::from)
	}

	pub fn mount_root(&mut self, root: Inode) {
		assert_eq!(self.n_mounts, 0, "Root filesystem already mounted");
		self.mount_inode(None, root);
	}

	pub fn mount(&mut self, path: &str, guest_root: Inode) {
		// TODO: Handle mount points that don't exist.
		// TODO: Mount with arbitrary relative paths instead of relative to
		//       `self.root()`.
		let host_inode = self
			.find(self.root(), path)
			.expect("Mount point doesn't exist");
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
		if self.n_mounts >= 4 {
			panic!("Too many filesystems mounted");
		}
		self.mounts[self.n_mounts] = MaybeUninit::new(MountPoint {
			host_inode_hash: host_inode.map(|inode| inode.hash()),
			guest_root_inode: guest_root,
		});
		self.n_mounts += 1;
	}

	fn mount_point(&self, node: &Inode) -> Option<Inode> {
		for i in 0..self.n_mounts {
			// Safety: n_mounts must represent the number of contiguously
			//         initialized mounts.
			let mp = unsafe { self.mounts[i].assume_init_ref() };
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
		n_mounts: 0,
		mounts: unsafe { MaybeUninit::uninit().assume_init() },
	})
}

pub fn fs0() -> &'static mut FileSystem {
	FS.get()
}
