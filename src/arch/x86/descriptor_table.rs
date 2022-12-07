use core::mem::size_of;

#[repr(packed)]
pub(super) struct DescriptorTableRegister {
	_limit: u16,
	_base: u32,
}

impl DescriptorTableRegister {
	pub fn new<T, const N: usize>(table: &[T; N]) -> Self {
		Self {
			_limit: size_of::<[T; N]>() as u16 - 1,
			_base: table as *const [T; N] as u32,
		}
	}
}
