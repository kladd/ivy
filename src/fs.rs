use crate::{fat::File, proc::Task};

pub fn open(path: &str) -> Option<File> {
	let task = unsafe { &*Task::current() };
	let fs = task.fs;

	fs.find(task.cwd, path).map(|node| fs.open(node))
}
